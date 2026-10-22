use super::*;

#[test]
fn a_storage_object_can_be_sized_and_mounted() {
    assert_eq!(opt("zfs:tank/data@quota=10G", "quota"), "10G");
    assert_eq!(opt("zfs:tank/data@mount=/srv", "mount"), "/srv");
    assert_eq!(opt("btrfs:/mnt/fs/data@quota=5G", "quota"), "5G");
    assert_eq!(opt("btrfs:/mnt/fs/data@mount=/srv", "mount"), "/srv");
    assert_eq!(
        opt(
            "btrfs:/mnt/fs/data@mount=/srv,mount_options=noatime",
            "mount_options"
        ),
        "noatime"
    );
}
