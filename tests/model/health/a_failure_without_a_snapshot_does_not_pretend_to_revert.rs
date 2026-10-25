use super::*;

/// Without a snapshot the failure is loud and nothing is pretended. This is the state the
/// pre-flight refusal exists to prevent ever reaching.
#[test]
fn a_failure_without_a_snapshot_does_not_pretend_to_revert() {
    let out = Outcome::of(vec!["boot".into()], None);
    assert!(matches!(out, Outcome::FailedWithoutRevert { .. }));
}
