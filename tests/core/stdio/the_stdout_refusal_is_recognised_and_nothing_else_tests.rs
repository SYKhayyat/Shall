use super::*;

/// The one panic that is a closed stdout.
///
/// The message is what the standard library formats inside `println!`, and it is the only thing
/// the hook has: the payload is a `String`, not an `io::Error`.
#[test]
fn the_standard_librarys_stdout_refusal_is_recognised() {
    assert!(is_closed_stdout(
        "failed printing to stdout: Broken pipe (os error 32)"
    ));
    // The Windows spellings, taken from the same format: a closed pipe and a pipe being closed.
    assert!(is_closed_stdout(
        "failed printing to stdout: The pipe is being closed. (os error 232)"
    ));
    assert!(is_closed_stdout(
        "failed printing to stdout: The pipe has been ended. (os error 109)"
    ));
}

/// The defect the old predicate had: **any** panic whose text mentioned a broken pipe was read
/// as "stdout closed", and the process left with `0`. A mutating run cut off halfway reported
/// success, which is the one thing an exit code must never say.
#[test]
fn a_broken_pipe_that_is_not_stdout_is_not_swallowed() {
    for message in [
        // A write to a child's stdin, through an `expect` that formatted the io::Error.
        "writing question to the installer: Broken pipe (os error 32)",
        "Broken pipe (os error 32)",
        // stderr's reader went away: the command's only output went nowhere, which is not a
        // delivered answer.
        "failed printing to stderr: Broken pipe (os error 32)",
        // A stdout write that failed for a different reason. Exiting `0` here would report a
        // full disk as a delivered answer.
        "failed printing to stdout: No space left on device (os error 28)",
        // Not about output at all.
        "index out of bounds: the len is 3 but the index is 5",
    ] {
        assert!(
            !is_closed_stdout(message),
            "`{message}` must not be read as a closed stdout"
        );
    }
}
