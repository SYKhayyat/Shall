use super::*;

/// The two refusals. Neither is a guess: an fstab entry for a path this backend cannot place
/// would mount the wrong object, and the filesystem root is the one line a machine boots by.
#[test]
fn a_path_that_is_not_under_a_btrfs_mount_has_no_fstab_entry() {
    let m = mounts("/dev/sdb1 /mnt/fs btrfs rw,subvol=/ 0 0\n");
    assert_eq!(subvol_arg(&m, "/home/me/data"), None, "another filesystem");
    assert_eq!(
        subvol_arg(&m, "/mnt/fs"),
        None,
        "the root of a mounted filesystem is not a subvolume this backend made, and writing \
         an entry for it would rewrite the line that mounts the machine"
    );
    assert_eq!(subvol_arg(&m, "/mnt/fsx/data"), None, "a name, not a path");
}
