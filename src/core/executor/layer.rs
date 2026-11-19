//! What a command actually *is* to Shall: the output shapes, the layer trait every
//! executor implements, and the process-wide bounds a child is held to.

use super::*;

/// Set on every process Shall spawns, carrying the pid of the Shall that spawned it. A
/// `shall` that finds it in its environment was started by a package manager Shall is
/// already driving.
pub const INSIDE_SHALL: &str = "SHALL_INSIDE";

#[derive(Debug, Clone, Default)]
pub struct DryRunOutput {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

/// An `ExitStatus` for a process that never ran.
///
/// `ExitStatus` has no public constructor, so this used to spawn `/bin/false` or `cmd /C exit 1`
/// and `.expect()` the result — two panics on a path whose whole point is that nothing runs, on
/// a host stripped enough not to have them. Both platforms expose the raw form; on Unix the raw
/// value is a wait status, where the exit code sits in the high byte.
/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a
/// test crate links the library without `cfg(test)`, so a private item is invisible there.
pub fn fabricate_status(code: i32) -> std::process::ExitStatus {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        // **A Unix exit code is eight bits**, so `.code()` reads back `(raw >> 8) & 0xff` and a
        // fabricated `0x8A150001` comes back as `1`. That is deliberate and demonstrated by
        // `a_fabricated_status_round_trips_only_what_this_os_can_hold`, which fabricates a
        // Windows code here on purpose to show it does not survive — so this cannot assert the
        // range without breaking the test that documents the rule.
        //
        // The rule was re-entered anyway, a third time, by a winget test carrying a 32-bit
        // HRESULT. It went unnoticed for a reason no assertion here would have fixed: the build
        // matrix produced one target out of four, so no test in this file had ever run on Linux.
        std::process::ExitStatus::from_raw(code << 8)
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::ExitStatusExt;
        std::process::ExitStatus::from_raw(code as u32)
    }
}

impl DryRunOutput {
    pub fn new() -> Self {
        Self::default()
    }

    /// A run that exited non-zero and complained — what a manager with no package index does.
    pub fn faulted(stderr: &str) -> StdOutput {
        StdOutput {
            status: fabricate_status(1),
            stdout: Vec::new(),
            stderr: stderr.as_bytes().to_vec(),
        }
    }
}

/// A run that exited non-zero and said nothing at all — what a manager that fell over before it
/// could speak leaves behind. `winget` under concurrent cold start is the measured case.
/// **`pub`, and it ships**, because the integration suite needs it and the library's own tests
/// need it too, and one fixture in two places is two of everything.
pub fn silent_failure(code: i32) -> StdOutput {
    StdOutput {
        status: fabricate_status(code),
        stdout: Vec::new(),
        stderr: Vec::new(),
    }
}

/// A run that exited non-zero and still produced its answer.
/// **`pub`, and it ships**, for the reason [`silent_failure`](Self::silent_failure) gives.
pub fn spoken_failure(code: i32, stdout: &str, stderr: &str) -> StdOutput {
    StdOutput {
        status: fabricate_status(code),
        stdout: stdout.as_bytes().to_vec(),
        stderr: stderr.as_bytes().to_vec(),
    }
}

impl From<DryRunOutput> for StdOutput {
    fn from(dry: DryRunOutput) -> Self {
        StdOutput {
            status: fabricate_status(0),
            stdout: dry.stdout,
            stderr: dry.stderr,
        }
    }
}

#[async_trait]
pub trait ExecutionLayer: Send + Sync {
    async fn execute(
        &self,
        cmd: &str,
        args: &[String],
        env: &HashMap<String, String>,
    ) -> Result<StdOutput>;
    fn check_command(&self, cmd: &str) -> bool;
    async fn symlink(&self, src: &Path, dst: &Path) -> Result<()>;

    /// Whether this layer's commands are the kind a person may have to answer — which is what
    /// makes a password prompt possible at all. **Not** whether the child is handed Shall's
    /// stdin: no manager's child is (`#132`), and the prompt belongs to
    /// `ensure_sudo_credentials`'s own spawn, which stays in Shall's process group.
    fn shares_stdin(&self) -> bool {
        false
    }
}

/// Whether a spawned child may read from Shall's own stdin.
///
/// It is the only stream a child ever shares. stdout and stderr are captured on every path,
/// because every read parses `output.stdout` — a child writing straight to the terminal hands
/// the parser an empty string while the user sees raw manager output and believes it worked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChildStdin {
    /// Reads and existence probes. There is nothing to type at them, and a read that consumed
    /// the terminal could not be answered anyway.
    Closed,
    /// Mutations. A person is at the terminal for them, so exactly one prompt is possible —
    /// `sudo -v`, primed once per run. The child is still given no terminal of its own (`#132`),
    /// and what it prints is mirrored to stderr as it arrives.
    Interactive,
}

/// How long a child may produce **nothing at all** before Shall stops waiting for it.
///
/// Silence, not duration. A `cargo install` compiling from source and an `apt dist-upgrade`
/// both run for tens of minutes and are working the whole time; no wall-clock cap can tell
/// them from a wedged one, because there is no number that is both above the first and below
/// the second. What separates them is that working commands *say* something — and the one
/// measured hang said nothing for 76 minutes while still holding its pipes open.
///
/// Seeded from `Config::command_idle_timeout_secs` at startup, where `0` means no bound.
static COMMAND_IDLE_TIMEOUT_SECS: once_cell::sync::OnceCell<u64> = once_cell::sync::OnceCell::new();

/// How long a **read** may produce nothing before Shall stops waiting for it.
///
/// The bound above was chosen for `Checkpoint-Computer`, a mutation that legitimately runs
/// silent for minutes, and reads inherited it because there was only one number. They are not
/// the same job: `winget list` takes 1.5s, `apt list --installed` under a second, and a
/// question that has gone quiet for a quarter of an hour is not about to answer. Fifteen
/// minutes of rope for a one-second question means a stuck read costs fifteen minutes to
/// learn what twenty seconds could have told you.
///
/// Seeded from `Config::query_idle_timeout_secs`, where `0` means no bound — and a bound
/// above `command_idle_timeout_secs` is pointless rather than wrong, since the outer one
/// fires first.
static QUERY_IDLE_TIMEOUT_SECS: once_cell::sync::OnceCell<u64> = once_cell::sync::OnceCell::new();

/// How many times a read that failed *transiently* is asked again before Shall gives up.
///
/// Reads are idempotent — that is the whole justification. A mutation retried on a guess can
/// install something twice; asking a manager what it has, twice, costs a second. Seeded from
/// `Config::read_retry_attempts`; `1` means ask once and do not retry.
static READ_RETRY_ATTEMPTS: once_cell::sync::OnceCell<u32> = once_cell::sync::OnceCell::new();

/// Above the longest legitimate silence anyone has measured, and far below the hang.
/// `Checkpoint-Computer` is the adversarial case: a real one is silent for its whole run.
pub const DEFAULT_COMMAND_IDLE_TIMEOUT_SECS: u64 = 900;

/// Set the process-wide command idle bound (called once during startup). Later calls no-op.
pub fn set_command_idle_timeout(secs: u64) {
    let _ = COMMAND_IDLE_TIMEOUT_SECS.set(secs);
}

pub(crate) fn command_idle_timeout() -> Option<std::time::Duration> {
    match *COMMAND_IDLE_TIMEOUT_SECS
        .get()
        .unwrap_or(&DEFAULT_COMMAND_IDLE_TIMEOUT_SECS)
    {
        0 => None,
        secs => Some(std::time::Duration::from_secs(secs)),
    }
}

/// Two minutes. The slowest read measured on any host here is a cold `winget list` at 2.6s
/// under sixteen-way contention, so this is ~46x the worst observed — wide enough that a fat
/// machine on a slow disk is never cut off, and narrow enough that a wedged question is a
/// two-minute wait instead of a fifteen-minute one.
pub const DEFAULT_QUERY_IDLE_TIMEOUT_SECS: u64 = 120;

/// Three: the measured failure is a cold-start collision that a warm winget does not
/// reproduce, so one more attempt is usually the whole fix and two more is the margin.
pub const DEFAULT_READ_RETRY_ATTEMPTS: u32 = 3;

/// How long Shall waits for a person to type a sudo password before giving up (`S88`).
///
/// Two minutes: long enough to find a password manager, short enough that an unattended
/// terminal is a two-minute pause and not a quarter-hour hang. It is deliberately **not** the
/// command idle bound: a package manager may legitimately work in silence for minutes, and a
/// password prompt cannot — either somebody is typing or nobody is there.
pub const DEFAULT_SUDO_PASSWORD_TIMEOUT_SECS: u64 = 120;

/// Seeded from `Config::sudo_password_timeout_secs`; `0` waits as long as sudo itself would.
static SUDO_PASSWORD_TIMEOUT_SECS: once_cell::sync::OnceCell<u64> =
    once_cell::sync::OnceCell::new();

/// Set the process-wide sudo password bound (called once during startup). Later calls no-op.
pub fn set_sudo_password_timeout(secs: u64) {
    let _ = SUDO_PASSWORD_TIMEOUT_SECS.set(secs);
}

pub(crate) fn sudo_password_timeout() -> Option<std::time::Duration> {
    match *SUDO_PASSWORD_TIMEOUT_SECS
        .get()
        .unwrap_or(&DEFAULT_SUDO_PASSWORD_TIMEOUT_SECS)
    {
        0 => None,
        secs => Some(std::time::Duration::from_secs(secs)),
    }
}

/// Set the process-wide read bounds (called once during startup). Later calls no-op.
pub fn set_query_bounds(idle_secs: u64, retry_attempts: u32) {
    let _ = QUERY_IDLE_TIMEOUT_SECS.set(idle_secs);
    let _ = READ_RETRY_ATTEMPTS.set(retry_attempts);
}

/// The bound a read waits under: its own, but never longer than the outer one, which fires
/// first anyway. A `0` on either means that one imposes nothing.
pub(crate) fn query_idle_timeout() -> Option<std::time::Duration> {
    let own = match *QUERY_IDLE_TIMEOUT_SECS
        .get()
        .unwrap_or(&DEFAULT_QUERY_IDLE_TIMEOUT_SECS)
    {
        0 => None,
        secs => Some(std::time::Duration::from_secs(secs)),
    };
    match (own, command_idle_timeout()) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (Some(a), None) => Some(a),
        (None, outer) => outer,
    }
}

pub(crate) fn read_retry_attempts() -> u32 {
    (*READ_RETRY_ATTEMPTS
        .get()
        .unwrap_or(&DEFAULT_READ_RETRY_ATTEMPTS))
    .max(1)
}
