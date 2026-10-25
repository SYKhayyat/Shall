use super::*;

#[test]
fn diff_reports_added_removed_and_version_changed() {
    // current: what's installed now; past: the snapshot we'd roll back to.
    let current = reg(&[("apt", "curl", "8.4"), ("cargo", "rg", "14.0")]);
    let past = reg(&[("apt", "curl", "8.2"), ("apt", "nano", "7.0")]);

    let diff = SnapshotRestore::calculate_diff(&current, &past);

    // In current but not past -> rolling back would REMOVE it.
    assert_eq!(diff.to_remove.len(), 1);
    assert_eq!(diff.to_remove[0].name, "rg");
    // In past but not current -> rolling back would RESTORE it.
    assert_eq!(diff.to_restore.len(), 1);
    assert_eq!(diff.to_restore[0].name, "nano");
    // In both, different version -> a version change.
    assert_eq!(diff.changed.len(), 1);
    let (cur, old) = &diff.changed[0];
    assert_eq!(cur.name, "curl");
    assert_eq!(cur.version.as_deref(), Some("8.4"));
    assert_eq!(old.version.as_deref(), Some("8.2"));
}
