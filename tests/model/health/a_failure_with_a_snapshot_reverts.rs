use super::*;

/// U7's ruling: one revert path. The outcome does not record which *kind* of check failed,
/// because the machine does not care — a broken boot and a broken nginx both mean go back.
#[test]
fn a_failure_with_a_snapshot_reverts() {
    let out = Outcome::of(vec!["apt:nginx".into()], Some("snap-1"));
    assert_eq!(
        out,
        Outcome::Revert {
            failed: vec!["apt:nginx".into()],
            snapshot: "snap-1".into()
        }
    );
}
