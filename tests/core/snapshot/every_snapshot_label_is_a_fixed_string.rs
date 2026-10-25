use super::*;

#[test]
fn every_snapshot_label_is_a_fixed_string() {
    // SEC5: the enum is the guard, so the set is closed and quote-free by construction.
    for l in [
        SnapshotLabel::PreSync,
        SnapshotLabel::PreUpgrade,
        SnapshotLabel::PurgeUndeclared,
        SnapshotLabel::PreCanary,
    ] {
        assert!(
            !l.as_str().contains('\'') && !l.as_str().is_empty(),
            "{} must be safe inside a single-quoted PowerShell string",
            l
        );
    }
}
