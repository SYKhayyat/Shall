//! Work that blocks a thread, run from a command that is `async`.
//!
//! **Shall's slowest waits are not on the network — they are on a person, or on a program.** A
//! confirm sits at a prompt until someone types; a TUI reads keys for as long as they browse;
//! `git commit` runs after every sync; a `btrfs subvolume snapshot` takes as long as it takes.
//! Every one of those was a plain blocking call reached straight from an `async fn`, which parks
//! a tokio worker for the whole of it. One worker of several is survivable rather than fatal,
//! which is exactly why it lasted this long — and why the next one was written the same way.
//!
//! Two shapes, two primitives:
//!
//! * [`on_the_terminal`] — waiting where the call cannot move: it owns the terminal, or its
//!   caller is synchronous. `block_in_place` moves the runtime's other tasks off this worker and
//!   lets the call stay where it is. [`command_output`] and [`command_status`] are that, spelled
//!   for the `std::process::Command` sites.
//! * [`off_the_runtime`] — work that *can* move, because nothing about it is tied to this
//!   thread: unpacking an archive, hashing a file, waiting out a file lock. That belongs on the
//!   blocking pool, where it neither parks a worker nor competes with one.

/// Wait where the call cannot move, without parking a runtime worker.
///
/// `block_in_place` panics on a current-thread runtime, and Shall builds one of those as a
/// fallback in `rhai_stdlib`, so the flavour is asked rather than assumed. Nothing reaches a
/// prompt from there today; a check that costs nothing is cheaper than a panic that depends on
/// that staying true — and cheaper still than one that only fires for whoever writes the hook
/// that does reach it.
pub fn on_the_terminal<T>(wait: impl FnOnce() -> T) -> T {
    use tokio::runtime::{Handle, RuntimeFlavor};
    match Handle::try_current().map(|h| h.runtime_flavor()) {
        Ok(RuntimeFlavor::MultiThread) => tokio::task::block_in_place(wait),
        // Either no runtime at all — a unit test, `main` before it starts one — or the
        // single-threaded fallback, where there is no other worker to protect.
        _ => wait(),
    }
}

/// Do blocking work on the blocking pool instead of on a runtime worker.
///
/// For work with no tie to this thread: `tar`, `flate2`, `xz2`, `zip`, a `sha2` pass over a
/// downloaded file, a `flock` wait. These are not milliseconds — a release tarball is seconds to
/// minutes, and the data-directory lock waits up to two — and they ran on the same worker that
/// was supposed to be driving everything else.
pub async fn off_the_runtime<T, F>(work: F) -> crate::core::Result<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    tokio::task::spawn_blocking(work)
        .await
        // A `JoinError` here is the work having panicked. It is reported rather than resumed,
        // because a panic that unwinds into a package manager's retry loop is a panic nobody
        // can attribute to the thing that caused it.
        .map_err(|e| crate::core::Error::Other(format!("a background step failed: {e}")))
}

/// Run a `std::process::Command` to completion without parking a runtime worker.
///
/// **The third door.** `core::executor`'s two are for `tokio` children, whose hazard is that
/// dropping the future detaches the process. A `std::process::Command` has the opposite shape:
/// it cannot be abandoned at all, because the call does not return until the child has exited —
/// which is precisely the problem. `git commit` after every sync, a `btrfs subvolume` snapshot,
/// a `--help` probe, an external vars provider: each of them held a worker for its whole run,
/// from an `async fn`, with nothing saying so.
///
/// These stay synchronous rather than becoming `tokio` children because their callers are
/// synchronous — `GitManager` is a sync API used from a dozen places — and rewriting those to
/// async to fix a threading problem would be a far larger change than the problem is.
pub fn command_output(
    command: &mut std::process::Command,
) -> std::io::Result<std::process::Output> {
    on_the_terminal(|| command.output())
}

/// The same, with a bound — for a child whose command line came from a *script*.
///
/// **[`command_output`] returns when the child does and never before**, which is the right
/// shape for `git commit` and for a `--help` probe and the wrong one for the two doors that run
/// user-authored code: an external `vars.<ext>` provider, and `sh()` in the Rhai stdlib. Both of
/// those sit at step 0 of resolution — before any package manager is asked and before any plan
/// exists — so they are on the path of *every* command, `check` and `list` and `plan` included.
/// A `vars.py` reading from a network mount that went away, or a `vars.shall` calling
/// `sh("git fetch")` against an unreachable remote, hung Shall for ever with no output and no
/// way out but Ctrl-C. On a scheduled run there is nobody to press it.
///
/// The reasoning was already written down twice — `events.rs` bounds a hook because it is
/// "arbitrary code fired by a timer, with nobody at the terminal", and `rhai_stdlib`'s
/// operation cap names this exact gap: *"it counts Rhai operations, not seconds — a hook whose
/// `sh()` runs for ten minutes is one."* This is the seconds.
///
/// A **whole-command** bound rather than the idle bound the `tokio` doors use: those watch a
/// stream they are already draining, and a synchronous child's output is drained by the reader
/// threads below, not by the waiter. `0` in `command_idle_timeout_secs` still means no bound,
/// so the escape hatch is the one users already know.
pub fn command_output_bounded(
    command: &mut std::process::Command,
    what: &str,
) -> std::io::Result<std::process::Output> {
    command_output_within(command, what, crate::core::executor::command_idle_timeout())
}

/// The body of [`command_output_bounded`] with the bound passed in.
///
/// Split out so a test can name its own: the process-wide one is a `OnceCell` seeded at
/// startup, and a test that set it would decide the value for every other test in the binary.
/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
/// crate links the library without `cfg(test)`, so a private item is invisible there.
pub fn command_output_within(
    command: &mut std::process::Command,
    what: &str,
    limit: Option<std::time::Duration>,
) -> std::io::Result<std::process::Output> {
    let Some(limit) = limit else {
        return command_output(command);
    };
    on_the_terminal(|| {
        use std::io::Read;
        use std::process::Stdio;
        let mut child = command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        // Drained on their own threads. A child that fills its output pipe stops writing, and a
        // waiter that is not reading would then be waiting on a child that is waiting on it —
        // a deadlock the timeout would report as a hang the script did not cause.
        //
        // Each thread hands its buffer to a channel when its `read_to_end` ends — which is at
        // pipe EOF, and EOF needs EVERY write-end closed, including any a leaked grandchild
        // holds. That is why the joins below are bounded: the tokio door measured this exact
        // shape ("a 20 s bound, a 64 s wall") and fixed it; this door hung forever past its own
        // deadline on one background child.
        let (tx, rx_out) = std::sync::mpsc::channel::<Vec<u8>>();
        let (tx_err, rx_err) = std::sync::mpsc::channel::<Vec<u8>>();
        let mut out = child.stdout.take();
        let mut err = child.stderr.take();
        std::thread::spawn(move || {
            let mut buf = Vec::new();
            if let Some(pipe) = out.as_mut() {
                let _ = pipe.read_to_end(&mut buf);
            }
            let _ = tx.send(buf);
        });
        std::thread::spawn(move || {
            let mut buf = Vec::new();
            if let Some(pipe) = err.as_mut() {
                let _ = pipe.read_to_end(&mut buf);
            }
            let _ = tx_err.send(buf);
        });

        use std::sync::mpsc::RecvTimeoutError;
        const DRAIN_GRACE: std::time::Duration = std::time::Duration::from_secs(5);

        let deadline = std::time::Instant::now() + limit;
        let status = loop {
            match child.try_wait()? {
                Some(status) => break status,
                None if std::time::Instant::now() >= deadline => {
                    // Killed rather than asked: this is not a package manager mid-transaction
                    // with a database to unwind, it is a script of the user's that stopped
                    // answering, and leaving it running is what "unbounded" already meant.
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::TimedOut,
                        format!(
                            "{} did not finish within {}s and was stopped. It runs before \
                             anything else Shall does, so a wait here is a wait on every \
                             command. Raise `command_idle_timeout_secs`, or set it to 0 to \
                             remove the bound.",
                            what,
                            limit.as_secs()
                        ),
                    ));
                }
                None => std::thread::sleep(std::time::Duration::from_millis(25)),
            }
        };
        // The child is done; the READERS may not be. Wait a short grace for the buffers, and
        // answer honestly if the pipes never close — the output is incomplete and pretending
        // otherwise would feed half an answer to whoever parses it.
        let drain =
            |rx: &std::sync::mpsc::Receiver<Vec<u8>>, stream: &str| -> std::io::Result<Vec<u8>> {
                match rx.recv_timeout(DRAIN_GRACE) {
                    Ok(buf) => Ok(buf),
                    Err(RecvTimeoutError::Timeout) => Err(std::io::Error::new(
                        std::io::ErrorKind::TimedOut,
                        format!(
                            "{what} exited, but its {stream} pipe never closed within {}s — a \
                         background process is holding it. Its output cannot be trusted and \
                         is discarded.",
                            DRAIN_GRACE.as_secs()
                        ),
                    )),
                    Err(RecvTimeoutError::Disconnected) => Ok(Vec::new()),
                }
            };
        let stdout = drain(&rx_out, "stdout")?;
        let stderr = drain(&rx_err, "stderr")?;
        Ok(std::process::Output {
            status,
            stdout,
            stderr,
        })
    })
}

/// The same, for a command whose streams are inherited and whose answer is its exit status.
pub fn command_status(
    command: &mut std::process::Command,
) -> std::io::Result<std::process::ExitStatus> {
    on_the_terminal(|| command.status())
}
