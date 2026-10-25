use super::*;

/// A mount table with no btrfs in it is not an error and not a subvolume — it is nothing.
/// This is the case on every Windows and macOS host that runs this suite.
#[test]
fn a_table_with_no_btrfs_yields_no_mounts() {
    assert!(btrfs_mounts_in("/dev/sda1 / ext4 rw 0 0\nproc /proc proc rw 0 0\n").is_empty());
    assert_eq!(
        btrfs_mounts_in("/dev/sdb1 /mnt/my\\040disk btrfs rw,subvol=/sub 0 0\n"),
        vec![BtrfsMount {
            device: "/dev/sdb1".to_string(),
            point: "/mnt/my disk".to_string(),
            prefix: "sub".to_string(),
        }],
        "an escaped space in a mount point is a space, and the subvol= prefix loses its slash"
    );
}
