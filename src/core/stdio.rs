//! What a run does when stdout closes under it.
//!
//! `println!` panics when the write fails, and the release profile is `panic = "abort"`, so a
//! closed pipe — `shall search x | head`, a `tee` that dies, an SSH session that drops — is a
//! core dump. `main` installs a hook that turns that one panic into an exit instead, and this
//! module is the part of the decision that can be read and tested on its own.
//!
//! **The hook used to answer "is this a broken pipe?" by substring, on any panic.** A write to a
//! child's stdin, or an `expect()` that formatted an `io::Error` of kind `BrokenPipe`, was read
//! as "stdout closed" and the process left with `0` — a mutating run that had been cut off
//! reporting success, which is the one thing an exit code must never say.
//!
//! **Two questions, and both have to be answered.** Whether the panic is the standard library
//! reporting stdout, and whether this run was changing the machine: the status that is right for
//! `shall search | head` is a lie for `shall sync | head`, because the wanted answer was
//! delivered in the first case and a transaction was interrupted in the second.

use std::sync::atomic::{AtomicBool, Ordering};

static CHANGES_THE_MACHINE: AtomicBool = AtomicBool::new(false);

/// Whether this run is one that writes, as `main` decided from the parsed command.
///
/// Set once, after the parse, from `Commands::writes` — the same answer the data lock is taken
/// on, so "mutating" means one thing in this program. A preview is not: `--dry-run` reaches
/// this module as `dry_run::active()` at the moment of the panic, and a preview that loses its
/// reader changed nothing, so it keeps the quiet exit a reader gets.
pub fn set_changes_the_machine(writes: bool) {
    CHANGES_THE_MACHINE.store(writes, Ordering::Relaxed);
}

/// Whether the run that is ending was changing the machine, and was not a preview.
pub fn changes_the_machine() -> bool {
    CHANGES_THE_MACHINE.load(Ordering::Relaxed) && !crate::core::dry_run::active()
}

/// Whether a panic payload is the standard library reporting a failed write to **stdout**
/// because the reader went away.
///
/// The message is the only thing available — `println!` panics with a formatted string, not
/// with the `io::Error` — so the match is on that format: `failed printing to stdout: Broken
/// pipe (os error 32)` on Unix, `... The pipe is being closed. (os error 232)` on Windows, and
/// `109` (`ERROR_BROKEN_PIPE`) for a reader that closed on the other side of a named pipe.
///
/// **`stderr` is deliberately not included.** `shall check 2>&1 | head` losing its reader is
/// not a closed stdout, and treating it as one would exit `0` for a command whose only output
/// went nowhere.
pub fn is_closed_stdout(message: &str) -> bool {
    message.starts_with("failed printing to stdout")
        && (message.contains("Broken pipe")
            || message.contains("os error 32")
            || message.contains("os error 109")
            || message.contains("os error 232"))
}

/// The status a run leaves when its stdout closed.
///
/// A run that changed nothing keeps `0`: the wanted output was delivered and there is nothing
/// left to report. A run that was changing the machine exits `1` — `U21` has no code for "was
/// interrupted", `Q3` refused to add a fifth, and `1` is the truthful reading of *Shall failed:
/// it could not carry the command out*.
pub fn closed_stdout_exit(changing: bool) -> crate::core::Exit {
    if changing {
        crate::core::Exit::Failed
    } else {
        crate::core::Exit::Converged
    }
}
