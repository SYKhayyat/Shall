use super::*;

/// The neighbours inside the family, which is where a shared table would have gone wrong: a
/// subvolume has no size to create at, a volume group has no quota, and ZFS keeps its mount
/// properties on the dataset rather than in fstab.
#[test]
fn the_storage_backends_do_not_share_each_others_options() {
    let err = format!("{}", p("zfs:tank/data@size=10G").unwrap_err());
    assert!(err.contains("`@size` is not an option on `zfs`"));
    assert!(
        err.contains("`@quota`"),
        "a refusal that does not name the option that does work is a puzzle: {}",
        err
    );
    assert!(p("btrfs:/mnt/fs/data@size=10G").is_err());
    assert!(p("lvm:vg0/data@quota=10G").is_err());
    assert!(p("lvm:vg0/data@mount=/srv").is_err());
    assert!(p("zfs:tank/data@mount_options=noatime").is_err());
}
