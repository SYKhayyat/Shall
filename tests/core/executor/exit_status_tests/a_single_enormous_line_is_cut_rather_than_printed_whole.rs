use super::*;

/// A stream with no newlines is one line, and one line has no cap without this. winget draws
/// its progress spinner with bare carriage returns, which `lines()` does not split on.
#[test]
fn a_single_enormous_line_is_cut_rather_than_printed_whole() {
    let stream = "x".repeat(9_000);
    let err = executor_for(ExitPolicy::default())
        .ensure_status("somepm", finished(1, "", &stream))
        .unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.chars().count() < 400,
        "{} characters reached the user from a single line",
        msg.chars().count()
    );
    assert!(msg.contains('…'), "nothing said it had been cut:\n{msg}");
}
