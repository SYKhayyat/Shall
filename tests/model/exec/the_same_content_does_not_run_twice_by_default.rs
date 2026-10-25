use super::*;

/// The exit condition's second clause: it does not run on the next sync.
#[test]
fn the_same_content_does_not_run_twice_by_default() {
    let d = Decision::of(&Verdict::Approved, 1, Ceiling::read(None));
    assert!(!d.will_run());
    assert!(matches!(d, Decision::AlreadyRun { count: 1, .. }));
}
