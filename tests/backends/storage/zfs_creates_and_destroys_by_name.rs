use super::*;

#[test]
fn zfs_creates_and_destroys_by_name() {
    assert_eq!(zfs_create("tank/data"), vec!["create", "tank/data"]);
    // Destroy is recursive: a dataset with children is one object, and the guard cleared the
    // whole of it.
    assert_eq!(zfs_destroy("tank/data"), vec!["destroy", "-r", "tank/data"]);
}
