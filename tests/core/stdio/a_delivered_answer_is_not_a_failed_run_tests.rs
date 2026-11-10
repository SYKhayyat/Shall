use super::*;

/// `shall search x | head` is the reason the hook exists: the wanted output was delivered, the
/// reader asked for one line, and there is nothing to report. It must stay `0` — a `set -o
/// pipefail` script that greps the first line of a report is not a failure.
#[test]
fn a_reader_that_asked_for_one_line_is_not_a_failure() {
    assert_eq!(closed_stdout_exit(false).code(), 0);
}

/// The other half: a run that was changing the machine when its stdout closed did not finish,
/// and `0` would say it had. `U21` has no code for "interrupted" and `Q3` refused to add a
/// fifth, so this is `1` — *Shall failed: it could not carry the command out*.
#[test]
fn a_run_that_was_changing_the_machine_failed() {
    assert_eq!(closed_stdout_exit(true).code(), 1);
    assert_eq!(
        closed_stdout_exit(true),
        shall::core::Exit::Failed,
        "the status must be one of the four published codes, not a new number"
    );
}
