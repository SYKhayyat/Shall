use crate::core::account::Account;
#[cfg(windows)]
use crate::core::launch::windows_effective_command;
use crate::core::launch::{describe, forget_path_lookups, program_exists};
use crate::core::sudo::SudoState;
use crate::core::supervise::{Stopping, LOCK_DIRS};
use crate::core::{Error, ExitPolicy, Result, Retryability};
use async_trait::async_trait;
use dashmap::DashMap;
use fs2::FileExt;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
#[cfg(windows)]
use std::process::Command as StdCommand;
pub use std::process::Output as StdOutput;
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::Mutex;
use tracing::debug;

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

    /// Whether a child spawned by this layer may read Shall's own stdin. Only the raw layer
    /// behind mutations may; a layer that spawns nothing answers false.
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
    /// Mutations. `sudo` asks for a password on the terminal it was started from, and a
    /// mutation that cannot ask for one cannot run.
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

fn sudo_password_timeout() -> Option<std::time::Duration> {
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
fn query_idle_timeout() -> Option<std::time::Duration> {
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

fn read_retry_attempts() -> u32 {
    (*READ_RETRY_ATTEMPTS
        .get()
        .unwrap_or(&DEFAULT_READ_RETRY_ATTEMPTS))
    .max(1)
}

pub struct RawExecutor {
    /// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a
    /// test crate links the library without `cfg(test)`, so a private item is invisible there.
    pub stdin: ChildStdin,
    pub idle: Option<std::time::Duration>,
}

impl RawExecutor {
    /// The layer behind `run_output`/`search_output`/`command_exists`.
    ///
    /// Bounded by `query_idle_timeout`, not the mutation bound. A question that has said
    /// nothing for two minutes is not about to answer, and making it wait out a bound sized
    /// for `Checkpoint-Computer` buys nothing but the wait.
    pub fn reader() -> Self {
        Self {
            stdin: ChildStdin::Closed,
            idle: query_idle_timeout(),
        }
    }

    /// The layer behind `run`/`run_exclusive`.
    ///
    /// U40 lets a mutation — and only a mutation — share stdin, because `sudo` asks for a
    /// password on the terminal it was started from. `run_on` never inserts `sudo` on Windows
    /// (`if sudo && !cfg!(windows)`), so no Windows mutation has that question to ask, while
    /// the shared terminal stayed and cost the full `command_idle_timeout_secs` any time a
    /// manager asked something else. Measured on one install: 48ms with stdin closed, 21.9s
    /// with a real console — at the shipped bound, a fifteen-minute silence ending in failure
    /// anyway. Closing it means the manager's own prompt is captured and reported instead.
    pub fn mutator() -> Self {
        Self {
            stdin: if cfg!(windows) {
                ChildStdin::Closed
            } else {
                ChildStdin::Interactive
            },
            idle: command_idle_timeout(),
        }
    }

    /// **Ships, and it is not C3's case.** The constructor mints no authority — it sets two
    /// fields an executor already has — so a test building an executor with an eager idle
    /// is a caller like any other, not one that has to stay behind `cfg(test)`.
    pub fn with_idle(stdin: ChildStdin, idle: Option<std::time::Duration>) -> Self {
        Self { stdin, idle }
    }

    /// How long a child gets to stop itself before it is killed outright.
    ///
    /// Long enough for a package manager to abort a transaction and unlink its lock, short
    /// enough that a run Shall has already given up on does not sit there. `dpkg`'s own
    /// shutdown path is the slowest of these and finishes well inside it.
    ///
    /// Unix only, because it is the grace between the signal that can be caught and the one that
    /// cannot, and Windows has only the second.
    #[cfg(unix)]
    pub(crate) const TERMINATION_GRACE: std::time::Duration = std::time::Duration::from_secs(10);

    /// Collect the child's output, optionally echoing it to the terminal as it arrives, and
    /// give up on a child that has gone silent.
    ///
    /// Both streams must be drained concurrently with the wait: a pipe buffer that fills while
    /// nothing reads it blocks the child forever, and a package manager writing more than the
    /// buffer holds is not an edge case.
    ///
    /// The wait is sliced rather than awaited whole so the child stays reachable between
    /// slices — killing it needs the same `&mut` the wait future holds.
    ///
    /// The child is stopped through [`Stopping`], never with a bare kill: what Shall spawns is
    /// usually a package manager, and how you stop one of those decides whether the machine is
    /// usable afterwards.
    pub(crate) async fn wait_watched(
        mut child: tokio::process::Child,
        cmd: &str,
        mirror: bool,
        idle: Option<std::time::Duration>,
    ) -> Result<StdOutput> {
        use std::sync::Mutex as SyncMutex;
        use std::time::Instant;
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        async fn pump<R: tokio::io::AsyncRead + Unpin>(
            mut src: R,
            mirror: bool,
            last: Arc<SyncMutex<Instant>>,
        ) -> std::io::Result<Vec<u8>> {
            let mut collected = crate::core::capture::Capped::new();
            let mut buf = [0u8; 8192];
            let mut sink = tokio::io::stderr();
            loop {
                let n = src.read(&mut buf).await?;
                *last.lock().unwrap_or_else(|e| e.into_inner()) = Instant::now();
                if n == 0 {
                    return Ok(collected.finish());
                }
                collected.push(&buf[..n]);
                // Reading never stops at the cap, only *keeping* does: a child whose output
                // nobody drained blocks on a full pipe forever, which is the failure the
                // concurrent drain above exists to prevent.
                if mirror {
                    sink.write_all(&buf[..n]).await?;
                    sink.flush().await?;
                }
            }
        }

        let last: Arc<SyncMutex<Instant>> = Arc::new(SyncMutex::new(Instant::now()));
        let out_pipe = child.stdout.take();
        let err_pipe = child.stderr.take();
        // From here the child is owned by the guard: every way out of this function — returning,
        // an error, or the whole future being dropped by an aborted worker — goes through it.
        let mut child = Stopping::new(child);
        let mut out_task = tokio::spawn({
            let last = last.clone();
            async move {
                match out_pipe {
                    Some(p) => pump(p, mirror, last).await,
                    None => Ok(Vec::new()),
                }
            }
        });
        let mut err_task = tokio::spawn({
            let last = last.clone();
            async move {
                match err_pipe {
                    Some(p) => pump(p, mirror, last).await,
                    None => Ok(Vec::new()),
                }
            }
        });

        let status = match idle {
            None => child.child.wait().await?,
            Some(idle) => loop {
                let quiet = last.lock().unwrap_or_else(|e| e.into_inner()).elapsed();
                let remaining = idle.saturating_sub(quiet);
                if remaining.is_zero() {
                    child.stop().await;
                    out_task.abort();
                    err_task.abort();
                    // Permanent: the retry loop would spend another `idle` per attempt on a
                    // command that has already proved it does not finish, and a user watching
                    // three silences instead of one learns nothing new from the second two.
                    return Err(Error::command_failed_permanently(format!(
                        "`{}` produced no output for {}s and had not exited; Shall asked it to \
                         stop and killed it if it would not. If this command is legitimately \
                         silent for longer, raise `command_idle_timeout_secs` (0 disables the \
                         bound).",
                        cmd,
                        idle.as_secs(),
                    )));
                }
                if let Ok(status) = tokio::time::timeout(remaining, child.child.wait()).await {
                    break status?;
                }
            },
        };
        // The child has exited; its *output* may not have. A manager that hands stdout to a
        // background process and returns leaves a pipe held by something outside this process
        // tree — `child.wait()` is done, there is nothing left to kill, and an unclocked
        // `await` here waits on an EOF that never comes. Measured: a 20s bound, a 64s wall,
        // and the install reported as a success. So the same clock keeps running over the
        // readers, on silence rather than duration, so a command still printing is never cut.
        let mut stalled = false;
        let out = Self::drain_watched(&mut out_task, &last, idle, &mut stalled).await;
        let err = Self::drain_watched(&mut err_task, &last, idle, &mut stalled).await;
        if stalled {
            out_task.abort();
            err_task.abort();
            return Err(Error::command_failed_permanently(format!(
                "`{}` exited, but something still holds its output open and has printed \
                 nothing for {}s; Shall stopped waiting. This is what a command that hands \
                 its work to a background process looks like from here. If it is legitimately \
                 silent for longer, raise `command_idle_timeout_secs` (0 disables the bound).",
                cmd,
                idle.map_or(0, |d| d.as_secs()),
            )));
        }
        let joined = |r: std::result::Result<std::io::Result<Vec<u8>>, tokio::task::JoinError>| {
            r.map_err(|e| Error::Other(format!("output reader failed: {}", e)))?
                .map_err(Error::from)
        };
        Ok(StdOutput {
            status,
            stdout: joined(out.expect("not stalled, so the reader finished"))?,
            stderr: joined(err.expect("not stalled, so the reader finished"))?,
        })
    }

    /// Await one output reader under the same silence bound the wait used.
    ///
    /// `stalled` latches: once either reader has gone quiet past the bound the other is not
    /// waited on at all, because the failure is already decided and waiting again would spend
    /// a second `idle` proving it.
    async fn drain_watched(
        task: &mut tokio::task::JoinHandle<std::io::Result<Vec<u8>>>,
        last: &Arc<std::sync::Mutex<std::time::Instant>>,
        idle: Option<std::time::Duration>,
        stalled: &mut bool,
    ) -> Option<std::result::Result<std::io::Result<Vec<u8>>, tokio::task::JoinError>> {
        if *stalled {
            return None;
        }
        let Some(idle) = idle else {
            return Some(task.await);
        };
        loop {
            let quiet = last.lock().unwrap_or_else(|e| e.into_inner()).elapsed();
            let remaining = idle.saturating_sub(quiet);
            if remaining.is_zero() {
                *stalled = true;
                return None;
            }
            if let Ok(done) = tokio::time::timeout(remaining, &mut *task).await {
                return Some(done);
            }
        }
    }
}

#[async_trait]
impl ExecutionLayer for RawExecutor {
    async fn execute(
        &self,
        cmd: &str,
        args: &[String],
        env: &HashMap<String, String>,
    ) -> Result<StdOutput> {
        // On Windows, route shim scripts (scoop's `.ps1`, `.cmd`/`.bat` wrappers) through
        // their interpreter so they can actually launch.
        #[cfg(windows)]
        let (eff_cmd, eff_args) = windows_effective_command(cmd, args);
        #[cfg(windows)]
        let (cmd, args) = (eff_cmd.as_str(), eff_args.as_slice());

        let mut command = Command::new(cmd);
        command.args(args).envs(env);

        // **Pinned off the directory Shall was invoked from.** A "global" install resolved
        // against the caller's CWD inherits whatever project files live there: a `.npmrc`
        // three directories up redirects the install, a `.cargo/config.toml` swaps the
        // registry, and the manager answers for a machine nobody configured on purpose.
        // Managers are invoked here in their machine-wide capacity or not at all, so they run
        // from the neutral temp directory — no project above it to be discovered by accident.
        {
            let neutral = std::env::temp_dir();
            if neutral.is_dir() {
                command.current_dir(neutral);
            }
        }

        // A worker whose task is aborted — a failed node, the global timeout — drops this
        // future, and dropping a future does not kill the process it spawned. Without this an
        // `apt install` keeps running against the same dpkg lock the rollback is about to take,
        // and whatever it completes is in no history that could compensate it.
        //
        // On Unix `Stopping` takes that job over, because tokio's version of it is SIGKILL and
        // **SIGKILL is not a way to stop a package manager** (see that type). Windows has no
        // gentler signal to send, so there it stays exactly as it was.
        #[cfg(windows)]
        command.kill_on_drop(true);
        #[cfg(unix)]
        {
            command.kill_on_drop(false);
            // Own process group: `Stopping` stops the tree, and a tree is only addressable if
            // the child leads its own group (`sudo` above, manager below, wrappers between).
            command.process_group(0);
        }

        command.stdout(Stdio::piped()).stderr(Stdio::piped());
        let interactive = self.stdin == ChildStdin::Interactive;
        command.stdin(if interactive && std::io::stdin().is_terminal() {
            Stdio::inherit()
        } else {
            // Not `inherit` when Shall's own stdin is a pipe: a child that reads it would eat
            // input meant for Shall, and one that blocks on it would never return.
            Stdio::null()
        });

        let child = command.spawn().map_err(|e| {
            let message = format!("Failed to spawn {}: {}", cmd, e);
            // A program that is not on PATH does not arrive during a backoff.
            if e.kind() == std::io::ErrorKind::NotFound {
                Error::command_failed_permanently(message)
            } else {
                Error::command_failed(message)
            }
        })?;

        // A mutation can run for minutes. Its progress used to reach the terminal because the
        // handles were inherited — which is exactly what emptied `output.stdout` and broke
        // every parser. Capture it and mirror it instead, so the bytes go both places.
        // The mirror is stderr, never stdout: stdout carries Shall's own answer, and a child's
        // chatter interleaved with it is not parseable by whoever piped us.
        let mirror = interactive && std::io::stderr().is_terminal();
        Self::wait_watched(child, &describe(cmd, args), mirror, self.idle).await
    }

    fn shares_stdin(&self) -> bool {
        self.stdin == ChildStdin::Interactive
    }

    fn check_command(&self, cmd: &str) -> bool {
        program_exists(cmd)
    }

    async fn symlink(&self, src: &Path, dst: &Path) -> Result<()> {
        #[cfg(unix)]
        {
            tokio::fs::symlink(src, dst)
                .await
                .map_err(|e| Error::Io(e.to_string()))
        }
        #[cfg(windows)]
        {
            if src.is_dir() {
                tokio::fs::symlink_dir(src, dst)
                    .await
                    .map_err(|e| Error::Io(e.to_string()))
            } else {
                tokio::fs::symlink_file(src, dst)
                    .await
                    .map_err(|e| Error::Io(e.to_string()))
            }
        }
    }
}

pub struct DryRunExecutor {
    vfs: Arc<DashMap<PathBuf, String>>,
}

impl DryRunExecutor {
    pub fn new(vfs: Arc<DashMap<PathBuf, String>>) -> Self {
        Self { vfs }
    }
}

#[async_trait]
impl ExecutionLayer for DryRunExecutor {
    async fn execute(
        &self,
        cmd: &str,
        args: &[String],
        _env: &HashMap<String, String>,
    ) -> Result<StdOutput> {
        crate::would!("Would execute: {} {}", cmd, args.join(" "));
        Ok(DryRunOutput::new().into())
    }

    /// Whether a command exists is a fact about this machine, not something a preview gets
    /// to invent. Answering `true` for everything made every backend look installed.
    fn check_command(&self, cmd: &str) -> bool {
        RawExecutor::reader().check_command(cmd)
    }

    async fn symlink(&self, src: &Path, dst: &Path) -> Result<()> {
        let val = format!("LINK:{}", src.display());
        self.vfs.insert(dst.to_path_buf(), val);
        Ok(())
    }
}

/// Strip inherited access and grant only the running user, via the tool Windows ships with.
///
/// Windows has no `mode` to create a file with, so "created restricted" is achieved by
/// restricting the temporary file and then renaming it into place — the destination never
/// exists in a readable state. A failure here is an error rather than a warning: the caller
/// is about to place a decrypted secret, and a secret that is not protected must not be
/// written at all (T5).
#[cfg(windows)]
fn restrict_to_owner(path: &Path) -> Result<()> {
    let user = std::env::var("USERNAME").map_err(|_| {
        Error::Refused(
            "cannot restrict the file: %USERNAME% is unset, so there is no account to grant \
             access to. Refusing to write a secret nothing protects."
                .into(),
        )
    })?;
    let output = StdCommand::new("icacls")
        .arg(path)
        .arg("/inheritance:r")
        .arg("/grant:r")
        .arg(format!("{}:F", user))
        .stdin(Stdio::null())
        .output()
        .map_err(|e| {
            Error::Other(format!(
                "could not run icacls to restrict {:?}: {}",
                path, e
            ))
        })?;
    if !output.status.success() {
        return Err(Error::Other(format!(
            "icacls could not restrict {:?}: {}",
            path,
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(())
}

/// The test double every mock-driven suite runs against.
///
/// **An unregistered command used to return `Ok(DryRunOutput::new())` — empty, success.** That
/// is the same "silence means fine" default the parser layer had, one storey up, and it does the
/// same damage: a test registers a stub, the product emits a slightly different argv, the stub is
/// never matched, the call falls through to the default, and the test passes having asserted
/// nothing.
///
/// It was not hypothetical. The since-deleted `e2e_tests.rs` registered `"brew install {name}"` while the
/// product emits `"brew install -- neovim"` (`argv.rs`); all five of that file's
/// registrations were dead strings and every test passed on the default. Sixteen lines away in
/// the same suite, `a_machine_converges_tests.rs` registered the `--` form. **Two tests
/// disagreed about the product's own argv and both were green.**
///
/// So the mock now keeps two ledgers, and the second is the one that catches that bug:
///
/// - **`unstubbed`** — commands that ran with no registration. Recorded rather than refused,
///   because a great many tests legitimately do not care what a command printed; they assert the
///   argv, or the state afterwards. **It was read by nothing at all**, so it was a record rather
///   than an instrument; it has a read and a per-test opt-in now. `V.215`.
/// - **`unmatched registrations`** — a stub that was set and never used. There is no innocent
///   reading of that: the test author wrote down what they expected the product to run, and the
///   product ran something else. It fails the test at drop.
///
/// The two are independent by construction, and the allowance for one is checked before the
/// other is consulted, so neither opt-in can switch off the other's check.
pub struct MockExecutor {
    pub responses: DashMap<String, Result<StdOutput>>,
    pub command_existence: DashMap<String, bool>,
    pub call_log: Arc<Mutex<Vec<String>>>,
    /// The environment the last call carried. The env map is where the pager suppression and
    /// the recursion guard live, and neither is visible in the argv the call log records.
    pub last_env: Arc<Mutex<HashMap<String, String>>>,
    /// Which registered patterns were actually matched by a call.
    matched: DashMap<String, ()>,
    /// Commands that ran with nothing registered for them, deduplicated.
    unstubbed: DashMap<String, ()>,
    /// Registrations whose whole purpose is to stay unmatched — see
    /// [`MockExecutor::set_response_that_must_not_be_used`].
    forbidden: DashMap<String, ()>,
    /// How long a command takes, for tests about concurrency rather than about output.
    delays: DashMap<String, std::time::Duration>,
    /// Set by a test that means it — see [`MockExecutor::allow_unmatched_registrations`].
    allow_unmatched: std::sync::atomic::AtomicBool,
    /// Set by a test that means it — see [`MockExecutor::deny_unstubbed`].
    deny_unstubbed: std::sync::atomic::AtomicBool,
    vfs: Arc<DashMap<PathBuf, String>>,
}

impl MockExecutor {
    pub fn new(vfs: Arc<DashMap<PathBuf, String>>) -> Self {
        Self {
            responses: DashMap::new(),
            command_existence: DashMap::new(),
            call_log: Arc::new(Mutex::new(Vec::new())),
            last_env: Arc::new(Mutex::new(HashMap::new())),
            matched: DashMap::new(),
            unstubbed: DashMap::new(),
            forbidden: DashMap::new(),
            delays: DashMap::new(),
            allow_unmatched: std::sync::atomic::AtomicBool::new(false),
            deny_unstubbed: std::sync::atomic::AtomicBool::new(false),
            vfs,
        }
    }

    pub fn set_response(&self, cmd_pattern: &str, response: Result<StdOutput>) {
        self.responses.insert(cmd_pattern.to_string(), response);
    }

    /// Make a command take time, so a test can observe two calls overlapping — or failing to.
    pub fn set_delay(&self, cmd_pattern: &str, delay: std::time::Duration) {
        self.delays.insert(cmd_pattern.to_string(), delay);
    }

    /// Register a convincing answer for a command the product must **not** run.
    ///
    /// **The foil, made into an assertion.** Five tests in this repo registered the *wrong*
    /// listing beside the right one — `dpkg-query -W` next to `apt-mark showmanual`, the
    /// essential query next to the manual one — to say *"and if adopt asks this instead, it will
    /// get a different set of packages and the assertion below will notice."* Every one of those
    /// stubs was dead, and its deadness was the whole point and was checked by nothing. Wire the
    /// product to the wrong listing and the test would have gone green, because the names it
    /// asserted happened to overlap.
    ///
    /// Registered this way, the stub answers if it is ever reached — so a product that asks the
    /// wrong question gets a wrong-shaped answer rather than empty success — and going unreached
    /// is asserted rather than assumed.
    pub fn set_response_that_must_not_be_used(
        &self,
        cmd_pattern: &str,
        response: Result<StdOutput>,
    ) {
        self.responses.insert(cmd_pattern.to_string(), response);
        self.forbidden.insert(cmd_pattern.to_string(), ());
    }

    /// Registered patterns no call ever matched.
    pub fn unmatched_registrations(&self) -> Vec<String> {
        let mut out: Vec<String> = self
            .responses
            .iter()
            .map(|e| e.key().clone())
            .filter(|k| !self.matched.contains_key(k) && !self.forbidden.contains_key(k))
            .collect();
        out.sort();
        out
    }

    /// For a test that registers a stub for a path it knows may not be taken — a fallback the
    /// product only reaches on another platform, or the second half of an either/or.
    ///
    /// Deliberately a per-mock opt-in with no default: the point of the check is that somebody
    /// has to look at the dead string and say it is meant to be dead.
    pub fn allow_unmatched_registrations(&self) {
        self.allow_unmatched
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    /// Every command that ran with nothing registered for it, deduplicated and sorted.
    ///
    /// **Append-only**: registering a response afterwards does not remove the record.
    pub fn unstubbed_registrations(&self) -> Vec<String> {
        let mut out: Vec<String> = self.unstubbed.iter().map(|e| e.key().clone()).collect();
        out.sort();
        out
    }

    /// Fail if any command ran with nothing registered for it.
    ///
    /// Per-mock, no default: 172 of the suite's tests leave a command unanswered, and 106 of
    /// those are existence probes whose `true` default is deliberate, so a global check would
    /// redden a quarter of the suite for having no opinion. Independent of
    /// [`MockExecutor::allow_unmatched_registrations`], and cannot be switched off by it.
    pub fn deny_unstubbed(&self) {
        self.deny_unstubbed
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    /// Fail if any registered stub went unused.
    ///
    /// Called from `Drop` so that a test does not have to remember; exposed because a test that
    /// wants the failure at a particular point reads better than one that gets it at the end of
    /// the scope.
    pub fn assert_every_registration_was_used(&self) {
        // The forbidden half first: a stub registered as one the product must not run, and run
        // anyway, is a louder failure than an unused one and must not be masked by it.
        let ran_anyway: Vec<String> = self
            .forbidden
            .iter()
            .map(|e| e.key().clone())
            .filter(|k| self.matched.contains_key(k))
            .collect();
        assert!(
            ran_anyway.is_empty(),
            "the product ran {} command(s) this test says it must not:

  {}

             Registered with `set_response_that_must_not_be_used`, which means the test's \
             claim is that this question is never asked.",
            ran_anyway.len(),
            ran_anyway.join(
                "
  "
            )
        );

        // Checked BEFORE the allowance below, and the order is load-bearing: one flag must not
        // be able to switch off the other half's check. `an_unanswered_command_is_a_defect_tests`
        // holds the case that would fail if these two lines were swapped.
        if self
            .deny_unstubbed
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            let unstubbed = self.unstubbed_registrations();
            if !unstubbed.is_empty() {
                let ran = self
                    .call_log
                    .try_lock()
                    .map(|l| l.clone())
                    .unwrap_or_default();
                panic!(
                    "{} command(s) ran with no mock response registered for them:\n\n  {}\n\n\
                     What actually ran:\n\n  {}\n\n\
                     Each fell through to the empty-success default, so every assertion this test \
                     makes about what Shall PRINTED proved nothing. Asserting the argv, or the \
                     state afterwards, is a fine reason to leave one unstubbed — but this test \
                     called `deny_unstubbed()`. Register the answer, or drop the call. `V.215`.",
                    unstubbed.len(),
                    unstubbed.join("\n  "),
                    if ran.is_empty() {
                        "(nothing)".to_string()
                    } else {
                        ran.join("\n  ")
                    }
                );
            }
        }

        if self
            .allow_unmatched
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            return;
        }
        let unused = self.unmatched_registrations();
        if unused.is_empty() {
            return;
        }
        let ran = self
            .call_log
            .try_lock()
            .map(|l| l.clone())
            .unwrap_or_default();
        panic!(
            "{} registered mock response(s) were never matched by any call:\n\n  {}\n\n\
             What actually ran:\n\n  {}\n\n\
             A stub nobody matched is the test's belief about the product's argv, and the \
             product disagreed. The call fell through to the empty-success default and the \
             assertions below it proved nothing — which is how the since-deleted `e2e_tests.rs` registered \
             `brew install {{name}}` against a product that emits `brew install -- neovim`, and \
             stayed green. Fix the pattern to the argv that ran, or call \
             `allow_unmatched_registrations()` and be able to say why.",
            unused.len(),
            unused.join("\n  "),
            if ran.is_empty() {
                "(nothing)".to_string()
            } else {
                ran.join("\n  ")
            }
        );
    }

    pub fn set_command_exists(&self, cmd: &str, exists: bool) {
        self.command_existence.insert(cmd.to_string(), exists);
    }

    pub async fn get_calls(&self) -> Vec<String> {
        self.call_log.lock().await.clone()
    }
}

#[async_trait]
impl ExecutionLayer for MockExecutor {
    async fn execute(
        &self,
        cmd: &str,
        args: &[String],
        env: &HashMap<String, String>,
    ) -> Result<StdOutput> {
        let full_cmd = format!("{} {}", cmd, args.join(" "));
        {
            let mut log = self.call_log.lock().await;
            log.push(full_cmd.clone());
        }
        {
            let mut seen = self.last_env.lock().await;
            *seen = env.clone();
        }
        // A command that takes time, for the tests that are about *when* two calls run rather
        // than what they return. Without it a mock command is instantaneous, so contention on
        // `run_exclusive`'s per-backend mutex is unobservable and a test of lock granularity
        // passes identically against one global mutex.
        if let Some(d) = self.delays.get(&full_cmd).map(|d| *d.value()) {
            tokio::time::sleep(d).await;
        }
        if let Some(res) = self.responses.get(&full_cmd) {
            self.matched.insert(full_cmd, ());
            return res.clone();
        }
        // Recorded, not refused. Most callers of an unstubbed command are asserting the argv or
        // the state afterwards and do not care what it printed; refusing here would redden them
        // all to catch the few that did care. `unmatched_registrations` is the half with no
        // innocent reading, and that one fails.
        self.unstubbed.insert(full_cmd, ());
        Ok(DryRunOutput::new().into())
    }

    /// Whether this command exists on the machine.
    ///
    /// Defaults to `true`, which is the same "silence means fine" choice as the empty output
    /// above and is kept for the same reason: a test that has not said otherwise is a test about
    /// something else, and a mock that answered `false` by default would have every backend
    /// report itself unavailable. The default is *recorded* so a test can ask.
    fn check_command(&self, cmd: &str) -> bool {
        match self.command_existence.get(cmd).map(|r| *r.value()) {
            Some(known) => known,
            None => {
                self.unstubbed.insert(format!("command -v {cmd}"), ());
                true
            }
        }
    }

    async fn symlink(&self, src: &Path, dst: &Path) -> Result<()> {
        let val = format!("LINK:{}", src.display());
        self.vfs.insert(dst.to_path_buf(), val);
        Ok(())
    }
}

impl Drop for MockExecutor {
    /// The check runs without a test having to remember it, which is the whole point: the tests
    /// that needed it were the ones whose authors did not know they did.
    ///
    /// Guarded on `thread::panicking` so a test already failing reports its own reason rather
    /// than this one — a second panic during unwind aborts the process and loses both messages.
    fn drop(&mut self) {
        if std::thread::panicking() {
            return;
        }
        self.assert_every_registration_was_used();
    }
}

/// What one read-and-retry pass produced. `Answerless` carries the classified error rather
/// than having been returned, so the caller — and only the caller — decides whether "ran,
/// failed, said nothing" is a failure or a declared silence.
pub enum ReadOutcome {
    Output(String),
    Answerless(Error),
}

/// **Cloning shares the run, it does not fork it.** Every field below is an `Arc`, so a clone
/// is another handle on one invocation's execution layer, dry-run filesystem, per-manager lock
/// map and installed-listing memo — which is what lets ~48 backends each hold one and still
/// contend on the same locks and answer from the same memo. There used to be a `duplicate()`
/// beside this that was `self.clone()` and nothing else, so the codebase said both; the two
/// names carried one meaning and neither carried this note.
#[derive(Clone)]
pub struct CommandExecutor {
    pub dry_run: bool,
    pub verbose: bool,
    pub inner: Arc<dyn ExecutionLayer>,
    /// Where questions go. A search or an existence probe changes nothing, so it runs for
    /// real even under `--dry-run`: stubbing it does not make the preview safer, it makes
    /// the preview wrong. `apt-cache search jq` answered from a stub is an empty answer,
    /// which reads as "apt does not have jq" and hands the name to whichever manager
    /// answers over the network instead.
    pub reader: Arc<dyn ExecutionLayer>,
    vfs: Arc<DashMap<PathBuf, String>>,
    lock_map: Arc<DashMap<String, Arc<Mutex<()>>>>,
    /// What this backend's manager means by its exit codes and its complaints. Empty until a
    /// backend claims one with [`with_exit_policy`](Self::with_exit_policy), which is what
    /// keeps manager names out of this file.
    exit_policy: Arc<ExitPolicy>,
    /// This run's installed listings, one per manager.
    ///
    /// Lives here rather than in a global because every backend of one `App` is built on a
    /// duplicate of that `App`'s executor and every duplicate shares this `Arc` — so the memo
    /// is scoped to the run, which is also what keeps one test's mock listing out of the next
    /// test in a suite where a hundred `App`s live in one process.
    installed: Arc<crate::core::installed::InstalledListings>,
    /// What this run has established about escalating, and the answer it shares with every
    /// clone of this executor.
    ///
    /// **Same shape as `installed`, same reason.** A run asks the machine for credentials once;
    /// the answer belongs to the run. It was a process-wide `static`, which made it the one
    /// piece of run state every test in the binary shared — and a test that wanted to watch the
    /// credential probe was at the mercy of whichever test answered first (`#121`).
    sudo: Arc<SudoState>,
}

impl CommandExecutor {
    pub fn new(dry_run: bool, verbose: bool) -> Self {
        let vfs = Arc::new(DashMap::new());
        let lock_map = Arc::new(DashMap::new());
        let inner: Arc<dyn ExecutionLayer> = if dry_run {
            Arc::new(DryRunExecutor::new(vfs.clone()))
        } else {
            Arc::new(RawExecutor::mutator())
        };
        Self {
            dry_run,
            verbose,
            inner,
            reader: Arc::new(RawExecutor::reader()),
            vfs,
            lock_map,
            exit_policy: Arc::new(ExitPolicy::default()),

            installed: Arc::new(crate::core::installed::InstalledListings::new()),
            sudo: Arc::new(SudoState::new()),
        }
    }

    pub fn with_layer(
        dry_run: bool,
        verbose: bool,
        layer: Arc<dyn ExecutionLayer>,
        vfs: Arc<DashMap<PathBuf, String>>,
        lock_map: Arc<DashMap<String, Arc<Mutex<()>>>>,
    ) -> Self {
        // A test injects one layer and expects to see every call on it, reads included.
        Self {
            dry_run,
            verbose,
            reader: layer.clone(),
            inner: layer,
            vfs,
            lock_map,
            exit_policy: Arc::new(ExitPolicy::default()),

            installed: Arc::new(crate::core::installed::InstalledListings::new()),
            sudo: Arc::new(SudoState::new()),
        }
    }

    /// Let this run reuse installed listings written by earlier runs, for `secs` (0 = never).
    ///
    /// Set from config after the executor exists, rather than taken as a constructor argument:
    /// `CommandExecutor::new` is called from twenty-odd tests that have no config and want the
    /// off default, and a third constructor parameter would have to be spelled out in all of
    /// them to say "the same as before".
    pub fn set_installed_cache(&mut self, secs: u64) {
        if secs > 0 {
            self.installed = Arc::new(crate::core::installed::InstalledListings::with_ttl(secs));
        }
    }

    /// Bind this manager's exit conventions to the executor the backend will run on.
    ///
    /// Called at registration, beside the rest of that backend's definition, so a manager
    /// with a new convention is added by declaring one — never by editing this file.
    /// Can this executor's manager tell Shall that a name does not exist?
    ///
    /// Exposed so the property is *observable*: a backend that quietly lost its policy still
    /// runs the same argv, so nothing else can see the loss. `cargo` and `pipx` lost theirs in
    /// the 2026-08-04 conversion with every argv assertion green.
    pub fn classifies_absent_names(&self) -> bool {
        !self.exit_policy.absent_markers.is_empty()
    }

    pub fn with_exit_policy(mut self, policy: ExitPolicy) -> Self {
        self.exit_policy = Arc::new(policy);
        self
    }

    /// Whether a command asking for privilege actually gets a `sudo` in front of it here.
    ///
    /// **Three environments, two answers, and a test cannot guess which.** Windows never
    /// escalates; Linux as an ordinary user does; Linux as root does not, because it is already
    /// there. A test that hard-codes one of those is a test that passes on the machine it was
    /// written on — which is exactly what happened: three `web` tests registered `dpkg -r fd`,
    /// passed on Windows for months, and were never run on Linux at all because the build matrix
    /// silently produced one target out of four. On a Linux runner the product ran
    /// `sudo dpkg -r fd` and the stub went unmatched.
    ///
    /// So the rule is a function, and the tests ask it rather than restating it. One rule, one
    /// place; a second copy is the copy that is wrong on somebody else's machine.
    pub fn escalates(sudo: bool) -> bool {
        sudo && !cfg!(windows) && !Self::is_root()
    }

    /// The argv `run_on` would actually launch, as one string — what a mock registers against.
    ///
    /// **Public, and not `#[cfg(test)]`, for the reason [`escalates`](Self::escalates) is a
    /// function**: the integration suite registers stubs by this exact string, and a stub that
    /// spells the prefix by hand is a stub that goes unmatched the next time the prefix moves.
    /// It moved when `S88` put `-n` on every escalated command, and four registrations in
    /// `backend_tests.rs` would have silently stopped matching on Linux — the platform this
    /// machine cannot run, which is how the last set of unmatched stubs survived for months.
    pub fn as_launched(cmd: &str, args: &[&str], sudo: bool) -> String {
        let line = std::iter::once(cmd)
            .chain(args.iter().copied())
            .collect::<Vec<_>>()
            .join(" ");
        if Self::escalates(sudo) {
            // `-n` is part of the argv, so it is part of what a mock matches (`S88`). Leaving it
            // out here would make every Linux stub register a command Shall no longer runs.
            format!("sudo -n {line}")
        } else {
            line
        }
    }

    pub fn is_root() -> bool {
        #[cfg(unix)]
        {
            unsafe { libc::geteuid() == 0 }
        }
        #[cfg(windows)]
        {
            false
        }
    }

    /// Run a command and return its raw output WITHOUT enforcing exit status. Reads and
    /// existence probes use this (directly or via `run_output`), because a non-zero exit
    /// is frequently a normal answer there — an empty search, a "not installed" query, an
    /// inactive service unit. Mutating callers must use `run`/`run_exclusive` instead.
    async fn run_raw(&self, cmd: &str, args: &[&str], sudo: bool) -> Result<StdOutput> {
        self.run_on(&self.inner, cmd, args, sudo, None).await
    }

    /// The same primitive, aimed at the layer that never stubs. Reads only.
    async fn read_raw(&self, cmd: &str, args: &[&str], sudo: bool) -> Result<StdOutput> {
        self.run_on(&self.reader, cmd, args, sudo, None).await
    }

    pub(crate) async fn run_on(
        &self,
        layer: &Arc<dyn ExecutionLayer>,
        cmd: &str,
        args: &[&str],
        sudo: bool,
        as_account: Option<&Account>,
    ) -> Result<StdOutput> {
        let escalate = Self::escalates(sudo);

        // `apt install`, run by a sync that already holds the data-directory lock, fires the
        // `DPkg::Post-Invoke` hook Shall installed — which is another `shall`, and it would
        // wait on a lock this process does not release until it exits. The env var travels
        // to every descendant, and `hook-reconcile` stands down when it sees it.
        let mut env = HashMap::new();
        env.insert(
            crate::core::executor::INSIDE_SHALL.to_string(),
            std::process::id().to_string(),
        );
        Self::suppress_pagers(&mut env);

        if let Some(account) = as_account {
            for (key, value) in account.session_env() {
                env.insert(key, value);
            }
        }
        let target = self.account_target(cmd, as_account)?;

        if escalate {
            self.ensure_sudo_credentials(layer).await?;
        }

        let (final_cmd, final_args) = Self::account_argv(cmd, args, escalate, target, &env);

        // Every manager invocation funnels through this one call, which is what makes
        // `--timings` a breakdown of the whole run rather than of whichever verbs remembered
        // to instrument themselves.
        let timing = crate::core::timing::begin();
        let result = layer.execute(&final_cmd, &final_args, &env).await;
        crate::core::timing::end(timing, &final_cmd, &final_args);
        result
    }

    pub(crate) fn is_benign_exit(&self, code: Option<i32>) -> bool {
        self.exit_policy.is_benign(code)
    }

    pub(crate) fn can_become(&self) -> bool {
        #[cfg(unix)]
        {
            Self::is_root() && self.command_exists_sync("sudo")
        }
        #[cfg(not(unix))]
        false
    }

    /// Stop a child from piping itself into a pager.
    ///
    /// `systemctl status`, `git log` and friends page when they believe a human is watching.
    /// A pager waits for a keypress that a captured child will never get, so the run hangs;
    /// and even when it does not, the escape sequences and the `lines 1-16/16 (END)` banner
    /// land in the text a parser is about to read. Capturing stdout removes the usual trigger,
    /// but `$PAGER`/`$SYSTEMD_PAGER` in the user's environment forces one anyway — so the
    /// suppression is set here, on the one env map every spawn inherits, rather than trusted
    /// to the absence of a terminal.
    /// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a
    /// test crate links the library without `cfg(test)`, so a private item is invisible there.
    pub fn suppress_pagers(env: &mut HashMap<String, String>) {
        // systemd reads an empty value as "no pager"; git and the rest need a command that
        // exists and exits, so `cat`.
        env.insert("SYSTEMD_PAGER".to_string(), String::new());
        env.insert("SYSTEMD_LESS".to_string(), String::new());
        env.insert("PAGER".to_string(), "cat".to_string());
        env.insert("GIT_PAGER".to_string(), "cat".to_string());
    }

    /// Run a *mutating* command and enforce success. `RawExecutor::execute` hands back the
    /// process output regardless of exit status, so without this a failed `apt remove` /
    /// `npm install` / `btrfs subvolume delete` would be silently reported as OK and the
    /// caller would trust a mutation that never actually happened. Callers that legitimately
    /// tolerate a non-zero exit (searches, existence probes) must use `run_output`/`run_raw`.
    pub async fn run(&self, cmd: &str, args: &[&str], sudo: bool) -> Result<StdOutput> {
        let output = self.run_raw(cmd, args, sudo).await?;
        let checked = self.ensure_status(cmd, output);
        // A mutation is the only thing that can change what is on PATH — or what is installed —
        // during a one-shot run. Installing `npm` and then asking whether `npm` is available is
        // a real sequence inside one `sync`. Every read stays memoised; a mutation is the edge
        // that invalidates them.
        self.forget_run_scoped_answers(None);
        checked
    }

    /// This run's installed listings, shared by every backend built on this executor.
    pub fn installed_listings(&self) -> &Arc<crate::core::installed::InstalledListings> {
        &self.installed
    }

    /// A read whose answer is its stdout.
    ///
    /// **A non-zero exit is tolerated; a non-zero exit that said nothing at all is not.** The
    /// distinction is the whole of it. "No such package" and "no results" are legitimate
    /// non-zero replies and they arrive with their reason on the page, so the exit code alone
    /// must never be the verdict — that is why this goes through the unchecked primitive rather
    /// than `run`. But a failure with nothing on *either* stream is not a reply at all, and
    /// returning `Ok("")` for it hands every caller an answer the manager never gave.
    ///
    /// Measured: 3 of 16 concurrent cold-start `winget list` exit `0x8A150001` having written
    /// zero bytes anywhere. Read as an empty listing, that made `shall list --backend winget`
    /// print nothing and exit 0 on a machine with 280 packages on it — and `info` report an
    /// installed package as absent, which is the shape it was first noticed in.
    ///
    /// **Retried when — and only when — the failure is classified transient.** Reads are
    /// idempotent, which is the whole justification: asking a manager what it has, twice,
    /// costs a second, where a mutation retried on a guess installs something twice. The
    /// measured case is a cold-start collision that a warm winget does not reproduce, so the
    /// second attempt is usually the entire fix.
    pub async fn run_output(&self, cmd: &str, args: &[&str], sudo: bool) -> Result<String> {
        match self.read_with_retry_as(None, cmd, args, sudo).await? {
            ReadOutcome::Output(s) => Ok(s),
            ReadOutcome::Answerless(e) => Err(e),
        }
    }

    /// [`Self::run_output`] with "the manager ran, failed, and said nothing" delivered as
    /// `Ok(None)` instead of an error — for the probes whose row declares that silence *is*
    /// the answer (`silence_is_none`).
    ///
    /// **The boundary is typed by construction, not matched out of prose.** The idle-timeout
    /// kill errors inside `read_with_retry` exactly as it does for `run_output`, so a wedged
    /// query killed at its bound arrives here as `Err` and propagates — it used to be
    /// indistinguishable from the answerless case because both said "no output", and a probe
    /// that timed out read as "zero updates", exit 0.
    pub async fn run_output_maybe_silent(
        &self,
        cmd: &str,
        args: &[&str],
        sudo: bool,
    ) -> Result<Option<String>> {
        match self.read_with_retry_as(None, cmd, args, sudo).await? {
            ReadOutcome::Output(s) => Ok(Some(s)),
            ReadOutcome::Answerless(_) => Ok(None),
        }
    }

    /// The one read-and-retry loop both output primitives share. An answerless read — non-zero
    /// exit, both streams empty — retries while transient and then becomes
    /// [`ReadOutcome::Answerless`], carrying the classified error for whoever decides whether
    /// that means failure or silence.
    pub(crate) async fn read_with_retry_as(
        &self,
        account: Option<&Account>,
        cmd: &str,
        args: &[&str],
        sudo: bool,
    ) -> Result<ReadOutcome> {
        let attempts = read_retry_attempts();
        let initial = std::time::Duration::from_millis(200);
        // The same cap the mutation retry uses (`TransactionConfig::max_backoff`): a cap is
        // not a mutation-only concern. Uncapped, ten attempts slept ~39s before the last
        // try — for a read, where the whole justification is that a second attempt usually
        // fixes it.
        let max = std::time::Duration::from_secs(30);
        let mut backoff = initial;
        for attempt in 1..=attempts {
            let output = match account {
                Some(account) => self.read_raw_as(account, cmd, args, sudo).await?,
                None => self.read_raw(cmd, args, sudo).await?,
            };
            let stdout = crate::utils::text::sanitize(&String::from_utf8_lossy(&output.stdout));
            let benign =
                output.status.success() || self.exit_policy.is_benign(output.status.code());
            // **Both streams, not just stdout.** A read that failed and *said* something has
            // described its own situation, and the caller may legitimately read that as an
            // empty result: `Get-ComputerRestorePoint` on an unelevated shell exits 1 with
            // `Access denied` on stderr and nothing on stdout, and treating that as fatal
            // failed a whole `sync` that had no business caring. Whether *that* is the right
            // answer is a separate question about snapshots, not about this primitive.
            //
            // Silence on both is the case with no second reading. Nothing expresses "you have
            // none of these" by saying nothing at all and failing.
            let said_nothing = stdout.trim().is_empty()
                && String::from_utf8_lossy(&output.stderr).trim().is_empty();
            if benign || !said_nothing {
                return Ok(ReadOutcome::Output(stdout));
            }
            let err = self.answerless_read(cmd, args, &output);
            if attempt == attempts || err.retryability() != Retryability::Transient {
                return Ok(ReadOutcome::Answerless(err));
            }
            debug!(
                "`{cmd}` produced no answer and the failure is transient; \
                 asking again ({attempt}/{attempts})"
            );
            tokio::time::sleep(backoff).await;
            backoff = std::cmp::min(backoff * 3, max);
        }
        unreachable!("the loop returns on its last attempt")
    }

    /// A read that must distinguish "this manager will not do that" from "it did it and found
    /// nothing" — the question `run_output` deliberately refuses to answer.
    ///
    /// Used to ask a manager for a machine-readable listing it may be too old to support
    /// (`Q43`). An unsupported flag exits non-zero with a usage message, which every other
    /// reader here hands back as an empty result — correct for them, and the exact bug `Q40`
    /// closed if a caller then treats it as the listing. So negotiation gets its own primitive
    /// rather than a looser rule for everybody.
    ///
    /// Not retried and not classified: the answer being sought *is* the failure.
    pub async fn probe_output(&self, cmd: &str, args: &[&str]) -> Result<String> {
        let output = self.read_raw(cmd, args, false).await?;
        if !output.status.success() && !self.exit_policy.is_benign(output.status.code()) {
            return Err(self.answerless_read(cmd, args, &output));
        }
        Ok(crate::utils::text::sanitize(&String::from_utf8_lossy(
            &output.stdout,
        )))
    }

    /// The failure for a read that exited non-zero without producing an answer.
    ///
    /// Carries the manager's own words when it left any, and its retryability, so the retry
    /// loop does not have to read the sentence back to decide what to do.
    pub(crate) fn answerless_read(&self, cmd: &str, args: &[&str], output: &StdOutput) -> Error {
        let code = output
            .status
            .code()
            .map(|c| c.to_string())
            .unwrap_or_else(|| "terminated by signal".to_string());
        let stderr = crate::utils::text::sanitize(&String::from_utf8_lossy(&output.stderr));
        let said = stderr
            .trim()
            .lines()
            .next()
            .unwrap_or("")
            .trim()
            .to_string();
        let detail = if said.is_empty() {
            String::new()
        } else {
            format!(" It said: {said}")
        };
        let hay = ExitPolicy::haystack(&output.stdout, &output.stderr);
        Error::CommandFailed {
            message: format!(
                "`{} {}` exited {code} with no output, so Shall has no answer from it — not an \
                 empty one.{detail}",
                cmd,
                args.join(" ")
            ),
            // By code as well as by text. This is the one failure whose haystack is reliably
            // empty, so the text lists cannot classify it and the code is the only signal
            // there is.
            retry: self.exit_policy.retryability_of(output.status.code(), &hay),
            absent_name: false,
        }
    }

    /// A read whose emptiness is an *answer*, so a command that could not produce one must
    /// say so instead of returning nothing.
    ///
    /// "This manager has no such package" and "this manager has no package index" both print
    /// nothing. Reading the second as the first is how a bare name walks past the manager
    /// that has it and freezes to a lower one (V.7c). A non-zero exit alone is not the
    /// signal — `pacman -Ss`, `dnf search` and `brew search` all exit non-zero for an
    /// ordinary empty result — so the fault is a non-zero exit *with* a complaint on stderr.
    pub async fn search_output(&self, cmd: &str, args: &[&str], sudo: bool) -> Result<String> {
        let output = self.read_raw(cmd, args, sudo).await?;
        let stderr = crate::utils::text::sanitize(&String::from_utf8_lossy(&output.stderr));
        let complaint = stderr.trim();
        if !output.status.success() && !complaint.is_empty() {
            let first = complaint.lines().next().unwrap_or(complaint);
            // Not `CommandFailed`: this sentence is read by a user, in a line that
            // already says which manager and which package, and "Command execution
            // failed:" in front of it is noise.
            return Err(Error::Other(format!(
                "`{}` could not answer: {}",
                cmd, first
            )));
        }
        Ok(crate::utils::text::sanitize(&String::from_utf8_lossy(
            &output.stdout,
        )))
    }

    /// The file behind `run_exclusive`'s cross-process lock, in Shall's own data directory.
    ///
    /// It lived at a fixed, guessable name in the shared temp directory and was opened with
    /// `File::create`, which truncates and follows symlinks — so anyone with write access to
    /// that directory could pre-plant `shall_apt.lock` as a symlink and have the next
    /// exclusive run, frequently privileged, truncate the target. `datalock.rs` had already
    /// solved this; this is the same treatment, so there is one locking style in the tree and
    /// not two.
    fn open_exec_lock(lock_key: &str) -> Result<File> {
        Self::open_lock_at(&crate::utils::safe_data_dir().join("exec-locks"), lock_key)
    }

    /// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a
    /// test crate links the library without `cfg(test)`, so a private item is invisible there.
    pub fn open_lock_at(dir: &Path, lock_key: &str) -> Result<File> {
        // Created once per process, not once per lock: this runs on every exclusive command,
        // for a directory that exists after the first one.
        if LOCK_DIRS.insert(dir.to_path_buf()) {
            crate::utils::file::ensure_dir(dir)?;
        }
        // A lock key is a backend name, and a backend name comes from a config file. Anything
        // that is not a plain name would otherwise pick the directory the lock lands in.
        let stem: String = lock_key
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        let path = dir.join(format!("{}.lock", stem));
        let open = || {
            OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(&path)
        };
        match open() {
            Ok(file) => Ok(file),
            // **The cache above is an optimisation, and it must not be load-bearing.** It records
            // that a directory was created, not that it still exists — and `create(true)` creates
            // the *file*, never its parent. So a data directory removed after the first exclusive
            // command of the process fails every one after it with a bare "cannot find the path
            // specified", which names neither the directory nor the fact that it went missing.
            //
            // Measured as a Windows CI flake: the data directory is a process-global setting, the
            // test suite points it at temporary directories, and a directory cached by one test
            // was deleted while another still held the memo. That is a test arrangement, but the
            // mechanism is not — `/tmp` reapers and cleanup tools do the same thing to a long
            // `sync`, and the answer in both cases is to make the directory again rather than to
            // trust a memo about the past.
            Err(_) => {
                crate::utils::file::ensure_dir(dir)?;
                open().map_err(Error::from)
            }
        }
    }

    pub async fn run_exclusive(
        &self,
        lock_key: &str,
        cmd: &str,
        args: &[&str],
        sudo: bool,
    ) -> Result<StdOutput> {
        let mutex = self
            .lock_map
            .entry(lock_key.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone();
        let _thread_guard = mutex.lock().await;

        if self.dry_run {
            return self.run(cmd, args, sudo).await;
        }

        let lock_file = Self::open_exec_lock(lock_key)?;
        // `fs2`'s flock is a blocking syscall. When a second `shall` holds this lock, taking it
        // inline parks a whole runtime worker for however long that other process runs —
        // minutes, for an `apt dist-upgrade`. The wait belongs on the blocking pool; the file
        // comes back out so the unlock below is the same handle.
        let lock_file =
            tokio::task::spawn_blocking(move || lock_file.lock_exclusive().map(|_| lock_file))
                .await
                .map_err(|e| Error::Io(format!("waiting for the `{}` lock: {}", lock_key, e)))?
                .map_err(Error::from)?;
        let result = self.run_raw(cmd, args, sudo).await;
        let _ = lock_file.unlock();
        // Same invalidation as `run`, and it has to be here too: this is the path most installs
        // and removals actually take, and it reaches `run_raw` directly rather than through
        // `run`. Without it the run-scoped listings would still be answering from before the
        // install — which `Prior` reads to decide what a rollback puts back.
        self.forget_run_scoped_answers(Some(lock_key));
        // Enforce status only after releasing the lock, so a failed mutation still frees it.
        self.ensure_status(cmd, result?)
    }

    /// Forget everything memoised for the length of a run that a mutation could have changed:
    /// what is on `PATH`, and what each manager has installed.
    pub(crate) fn forget_run_scoped_answers(&self, manager: Option<&str>) {
        forget_path_lookups();
        self.installed.forget_after_mutation(manager);
    }

    /// What a failed command's own output is allowed to put on a terminal.
    ///
    /// A manager's stream is untrusted text of unbounded length: one `scoop` typo produced ~110
    /// lines of unrelated bucket commits with raw SGR sequences, and the sentence that mattered
    /// — `Couldn't find manifest for 'x'` — was the fourth of them. The escapes were already
    /// handled for the *machine* (`ExitPolicy::opening` strips them so detection works) and not
    /// for the person, which is the wrong way round.
    ///
    /// So: the manager's own vocabulary picks the lines that explain the failure; failing that,
    /// the tail, because a tool that says nothing it declared usually ends with its complaint.
    /// Whatever is dropped is named and reachable — the whole stream is logged at `-v` by the
    /// caller, and a count with nowhere to look is not one place to look.
    /// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a
    /// test crate links the library without `cfg(test)`, so a private item is invisible there.
    pub fn detail_for_user(policy: &ExitPolicy, stream: &str) -> String {
        /// Enough for a stack trace's opening, far short of a bucket update.
        const CAP: usize = 8;

        let lines: Vec<&str> = stream
            .lines()
            .map(str::trim_end)
            .filter(|l| !l.trim().is_empty())
            .collect();
        if lines.is_empty() {
            return String::new();
        }
        let explaining = policy.explaining_lines(stream);
        let chosen: Vec<&str> = if !explaining.is_empty() {
            explaining.into_iter().take(CAP).collect()
        } else if lines.len() > CAP {
            lines[lines.len() - CAP..].to_vec()
        } else {
            lines.clone()
        };
        /// A line long enough for any sentence a manager writes, and short enough that a stream
        /// with no newlines in it — a progress bar drawn with bare carriage returns is one long
        /// line to `lines()` — cannot fill a terminal past the cap above.
        const WIDTH: usize = 240;

        let shown: Vec<String> = chosen
            .iter()
            // A tab is a column, not an escape: kept as spaces so a manager's table still reads
            // as one. Everything else that moves a cursor or reverses a line is named by
            // codepoint, by the same function the grammar's refusals use.
            .map(|l| crate::core::validator::printable(&l.replace('\t', "    ")))
            .map(|l| {
                if l.chars().count() > WIDTH {
                    let head: String = l.chars().take(WIDTH).collect();
                    format!("{head}…")
                } else {
                    l
                }
            })
            .collect();
        let dropped = lines.len().saturating_sub(shown.len());
        let mut out = shown.join("\n");
        if dropped > 0 {
            out.push_str(&format!(
                "\n({} more line(s) of output; re-run with -v to see all of it)",
                dropped
            ));
        }
        out
    }

    /// Classify a finished mutating command as success or failure, and — when it failed —
    /// whether another attempt could go differently.
    ///
    /// What a non-zero exit or a zero-exit complaint means belongs to the manager, so it is
    /// read off this executor's [`ExitPolicy`] rather than matched on the program's name
    /// here. An executor with no policy is the honest default: every non-zero exit fails and
    /// nothing is classified, which is what the retry loop already assumed.
    /// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a
    /// test crate links the library without `cfg(test)`, so a private item is invisible there.
    pub fn ensure_status(&self, cmd: &str, output: StdOutput) -> Result<StdOutput> {
        let status_ok = output.status.success() || self.exit_policy.is_benign(output.status.code());
        // One lowercased join of both streams for all three marker questions below. Each used
        // to build its own, so a command's whole transcript was copied and lowercased three
        // times — per package, and an `apt install` or `cargo build` transcript is not small.
        // A policy with no markers answers every question from its empty lists and never reads
        // this, so it is not built for one.
        let hay = if self.exit_policy.reads_output() {
            ExitPolicy::haystack(&output.stdout, &output.stderr)
        } else {
            String::new()
        };
        if status_ok && !self.exit_policy.signals_failure(&hay) {
            return Ok(output);
        }
        let code = output
            .status
            .code()
            .map(|c| c.to_string())
            .unwrap_or_else(|| "terminated by signal".to_string());
        // scoop's failure marker lands on stdout, not stderr, so fall back to stdout for
        // the diagnostic when stderr is empty (e.g. a `status_ok` malignant-success case).
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = crate::utils::text::sanitize(&String::from_utf8_lossy(&output.stdout));
        let stream = {
            let e = stderr.trim();
            if e.is_empty() {
                stdout.trim()
            } else {
                e
            }
        };
        // Everything the command said, for whoever asked for the internals. The message below
        // keeps only what explains the failure, so this is the one place the rest survives.
        if !stream.is_empty() {
            debug!("`{}` said:\n{}", cmd, stream);
        }
        let detail = Self::detail_for_user(&self.exit_policy, stream);
        // A manager that exited 0 and said it failed is not described by its exit code.
        // "`scoop` failed (exit 0)" is a sentence that argues with itself, and it was the
        // first thing a user saw after a typo; when the verdict comes from what the command
        // printed, the message says that instead of quoting a status that means the opposite.
        let verdict = if output.status.success() {
            format!("`{}` reported a failure", cmd)
        } else {
            format!("`{}` failed (exit {})", cmd, code)
        };
        let msg = if detail.is_empty() {
            verdict
        } else {
            format!("{}: {}", verdict, detail)
        };
        Err(Error::CommandFailed {
            message: msg,
            retry: self.exit_policy.retryability(&hay),
            // Asked of the whole output, here, because this is the last place it exists: the
            // message keeps whichever stream was non-empty, so a marker on the other one is
            // gone by the time any caller sees it.
            absent_name: self.exit_policy.names_an_absent_package(&hay),
        })
    }

    pub async fn read_file(&self, path: &Path) -> Result<String> {
        if self.dry_run {
            if let Some(content) = self.vfs.get(path) {
                return Ok(content.clone());
            }
        }
        tokio::fs::read_to_string(path).await.map_err(Error::from)
    }

    /// Write a file the *machine* owns — a systemd unit, a `link:` target, a backend's state
    /// file — atomically and durably.
    ///
    /// The executor's preview policy, not [`crate::utils::file::persist`]'s: a dry run diverts
    /// the bytes into the VFS so a later read in the same run sees what this run would have
    /// written. Both policies are legitimate and they answer different questions; only the
    /// durability is shared.
    ///
    /// **It had no `flush` and no `sync_all`.** A rename is atomic against a concurrent reader
    /// and says nothing about power loss, so a crash after a sync could leave a zero-length
    /// systemd unit while `registry.json` and the WAL — which go through `persist` — survived
    /// intact. Shall's record of what it did, without the thing it did.
    pub async fn write_atomic(&self, path: &Path, content: &str) -> Result<()> {
        if self.dry_run {
            self.vfs.insert(path.to_path_buf(), content.to_string());
            return Ok(());
        }
        Self::durably(path, content, |_| Ok(())).await
    }

    /// Remove a file Shall placed, recording nothing in a dry run.
    ///
    /// A dry run does not delete, and it also does not *pretend* to: inserting a marker into
    /// the VFS would make later dry-run reads of this path answer from fiction. Already-gone
    /// is done, which is what makes teardown idempotent.
    pub async fn remove_file(&self, path: &Path) -> Result<()> {
        if self.dry_run {
            return Ok(());
        }
        match tokio::fs::remove_file(path).await {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    /// The one place this crate's async writers meet the one durable write.
    ///
    /// `spawn_blocking` because `sync_all` parks a thread on the disk, and parking a runtime
    /// worker there stalls every other task on it.
    async fn durably(
        path: &Path,
        content: &str,
        prepare: impl FnOnce(&Path) -> Result<()> + Send + 'static,
    ) -> Result<()> {
        let path = path.to_path_buf();
        let content = content.to_string();
        tokio::task::spawn_blocking(move || {
            crate::utils::file::durable_write(&path, &content, prepare)
        })
        .await
        .map_err(|e| Error::Other(format!("IO thread failure: {}", e)))?
    }

    /// Write content that must never be world-readable, restricted **before** it reaches its
    /// destination (T5).
    ///
    /// The restriction is applied to the temporary file and the file is then renamed into
    /// place, so there is no instant at which the target path holds readable plaintext. A
    /// chmod after the write would be that instant, however short — and a secret is exactly
    /// the file where "however short" is not an argument.
    ///
    /// On Unix the temp file is already created `0600` by `tempfile`; this asserts it rather
    /// than assuming it. On Windows the inherited ACEs are stripped and only the running user
    /// is granted access, via `icacls` — Shall drives the tool the OS already has.
    pub async fn write_secret(&self, path: &Path, content: &str) -> Result<()> {
        if self.dry_run {
            self.vfs.insert(path.to_path_buf(), content.to_string());
            return Ok(());
        }
        Self::durably(path, content, |temp| {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(temp, std::fs::Permissions::from_mode(0o600))
                    .map_err(Error::from)?;
            }
            #[cfg(windows)]
            {
                restrict_to_owner(temp)?;
            }
            Ok(())
        })
        .await
    }

    pub async fn symlink(&self, src: &Path, dst: &Path) -> Result<()> {
        self.inner.symlink(src, dst).await
    }

    pub fn get_vfs_diff(&self) -> Vec<(PathBuf, String)> {
        self.vfs
            .iter()
            .map(|item| (item.key().clone(), item.value().clone()))
            .collect()
    }

    pub async fn command_exists(&self, cmd: &str) -> bool {
        self.reader.check_command(cmd)
    }

    pub fn command_exists_sync(&self, cmd: &str) -> bool {
        self.reader.check_command(cmd)
    }

    /// Make sure `sudo` will run without asking, or fail **now** with a sentence saying why
    /// (`S88`).
    ///
    /// **The bug this ends.** sudo reads its password from `/dev/tty`, not from stdin, so
    /// nothing Shall did to the child's stdin could stop it waiting: a wrong password left the
    /// program sitting for 900 seconds, and so did a terminal with nobody in front of it. Both
    /// were silent — no prompt reached the user, because the child's stderr is captured — and
    /// both looked exactly like a slow package manager. It failed that way every night for six
    /// nights in the `tools` leg, and took 48 minutes to do it.
    ///
    /// **Three outcomes, all of them prompt.** The timestamp is already warm, so nothing is
    /// asked; a person is at a terminal and gets one bounded chance to type; or there is nobody
    /// to ask and Shall says so immediately instead of waiting to be told what it already knows.
    async fn ensure_sudo_credentials(&self, layer: &Arc<dyn ExecutionLayer>) -> Result<()> {
        if cfg!(windows) || Self::is_root() || self.dry_run {
            return Ok(());
        }
        if self.sudo.is_primed() {
            return Ok(());
        }
        // **The answer is remembered in both directions** (`S89`). A refusal is as permanent
        // within one run as a success is, and asking again costs the whole password bound again
        // — per escalated command, on the verbs that deliberately keep going after one backend
        // fails. That is how a 120-second bound produced a 900-second wedge.
        if let Some(why) = self.sudo.refusal() {
            return Err(Error::command_failed_permanently(why));
        }

        // Everything above is a fast path off an already-settled answer. Past here the call
        // may spawn a probe and may put a prompt on the terminal, so only one task does it —
        // and both cells are re-read after the wait, because the task that held the lock has
        // just written whichever of them applies.
        let _priming = self.sudo.priming().await;
        if self.sudo.is_primed() {
            return Ok(());
        }
        if let Some(why) = self.sudo.refusal() {
            return Err(Error::command_failed_permanently(why));
        }
        // Warm timestamp, `NOPASSWD`, or an already-primed session — and **through the layer, not
        // around it**: this spawned `Command::new("sudo")`, the one command `with_layer` could not
        // see, so an escalation test needed passwordless sudo on the host to pass (`#88`). A layer
        // can now *answer* the probe.
        let probe = ["-n".to_string(), "-v".to_string()];
        let warm = layer.execute("sudo", &probe, &HashMap::new()).await;
        if matches!(warm, Ok(out) if out.status.success()) {
            self.sudo.mark_primed();
            return Ok(());
        }

        // Nobody to ask. **Permanent, not transient**: a password does not arrive during a
        // backoff, and retrying spends the whole bound again to learn the same thing.
        if !(layer.shares_stdin() && std::io::stdin().is_terminal()) {
            return Err(self.sudo.refuse(
                "sudo needs a password and there is no terminal to ask on. Run Shall from a \
                 terminal, give this user a NOPASSWD rule for the package managers it drives, \
                 or run it as root."
                    .to_string(),
            ));
        }

        let ask = Command::new("sudo")
            .arg("-v")
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .kill_on_drop(true)
            .status();
        let answered = match sudo_password_timeout() {
            Some(bound) => match tokio::time::timeout(bound, ask).await {
                Ok(status) => status,
                Err(_) => {
                    return Err(self.sudo.refuse(format!(
                        "sudo asked for a password and none was entered within {}s. Set \
                         `sudo_password_timeout_secs` to wait longer, or `0` to wait as long as \
                         sudo would.",
                        bound.as_secs()
                    )))
                }
            },
            None => ask.await,
        };
        match answered {
            Ok(status) if status.success() => {
                self.sudo.mark_primed();
                Ok(())
            }
            // sudo has already printed its own reason — a wrong password, an account that is
            // not a sudoer — on the terminal it owns. Repeating a guess at it here would be
            // narration over a message the user has already read.
            Ok(_) => Err(self
                .sudo
                .refuse("sudo refused: Shall cannot run the commands that need root.".to_string())),
            Err(e) => Err(self.sudo.refuse(format!("sudo could not be run: {e}"))),
        }
    }

    /// Refresh the `sudo` timestamp for as long as the returned guard is held, so a long sync
    /// is not interrupted halfway by a password prompt.
    ///
    /// The guard is the whole point: the previous version handed back a bare `JoinHandle`,
    /// which detaches when dropped, so the loop outlived every caller and could not be
    /// stopped by any of them.
    pub async fn start_sudo_keepalive(&self) -> SudoKeepalive {
        if cfg!(windows) || Self::is_root() || self.dry_run {
            return SudoKeepalive(None);
        }
        // A machine that has already recorded a permanent refusal will not start answering
        // `sudo -n -v`, so refreshing a timestamp that does not exist is one pointless process
        // every sixty seconds for the rest of the run.
        if self.sudo.refusal().is_some() {
            return SudoKeepalive(None);
        }
        SudoKeepalive(Some(tokio::spawn(async move {
            loop {
                // `-n` so this never prompts. The foreground command owns the terminal; a
                // background task racing it for the same password prompt is two processes
                // reading one keyboard, and the visible one loses. An expired timestamp is
                // the foreground command's to raise, where the user can see which command
                // is asking.
                let _ = Command::new("sudo")
                    .args(["-n", "-v"])
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .kill_on_drop(true)
                    .status()
                    .await;
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            }
        })))
    }
}

/// Stops the `sudo -v` loop when it goes out of scope.
/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a
/// test crate links the library without `cfg(test)`, so a private item is invisible there.
pub struct SudoKeepalive(pub Option<tokio::task::JoinHandle<()>>);

impl SudoKeepalive {
    /// Whether a refresher is actually running — false on Windows, as root, and under
    /// `--dry-run`, where there is no timestamp to keep warm.
    pub fn is_running(&self) -> bool {
        self.0.as_ref().is_some_and(|h| !h.is_finished())
    }
}

impl Drop for SudoKeepalive {
    fn drop(&mut self) {
        if let Some(handle) = self.0.take() {
            handle.abort();
        }
    }
}
