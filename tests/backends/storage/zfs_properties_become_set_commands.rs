use super::*;

#[test]
fn zfs_properties_become_set_commands() {
    assert_eq!(
        zfs_set("quota", "10G", "tank/data"),
        vec!["set", "quota=10G", "tank/data"]
    );
    assert_eq!(
        zfs_set("mountpoint", "/mnt/data", "tank/data"),
        vec!["set", "mountpoint=/mnt/data", "tank/data"]
    );
}
