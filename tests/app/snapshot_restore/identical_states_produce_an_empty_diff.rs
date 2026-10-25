use super::*;

#[test]
fn identical_states_produce_an_empty_diff() {
    let a = reg(&[("apt", "curl", "8.4")]);
    let b = reg(&[("apt", "curl", "8.4")]);
    let diff = SnapshotRestore::calculate_diff(&a, &b);
    assert!(diff.to_remove.is_empty() && diff.to_restore.is_empty() && diff.changed.is_empty());
}
