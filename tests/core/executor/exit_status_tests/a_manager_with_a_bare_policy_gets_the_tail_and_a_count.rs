use super::*;

/// A manager with nothing declared still has to be legible: no markers, so no line can be
/// singled out, and the tail is what a tool that failed usually ends with.
#[test]
fn a_manager_with_a_bare_policy_gets_the_tail_and_a_count() {
    let stream: String = (0..40).map(|i| format!("line {i}\n")).collect();
    let err = executor_for(ExitPolicy::default())
        .ensure_status("somepm", finished(1, "", &stream))
        .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("line 39"), "the tail is missing:\n{msg}");
    assert!(!msg.contains("line 0\n"), "the head was kept:\n{msg}");
    assert!(msg.contains("32 more line(s)"), "no count:\n{msg}");
}
