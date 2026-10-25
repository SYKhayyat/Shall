use super::*;

/// Moving a declared mount has to release the old one. btrfs holds a subvolume at as many
/// places as you ask, so writing the new fstab entry and mounting it would leave the machine
/// mounted somewhere no declaration names, until a reboot cleared it.
#[test]
fn every_live_mount_of_a_subvolume_is_found() {
    let table = "/dev/sdb1 /mnt/fs btrfs rw,subvol=/ 0 0\n\
                 /dev/sdb1 /srv btrfs rw,subvol=/data 0 0\n\
                 /dev/sdb1 /old btrfs rw,subvol=/data 0 0\n\
                 /dev/sdb1 /other btrfs rw,subvol=/elsewhere 0 0\n";
    let core = core_with(table, &[]);
    let mut points = core.current_mounts_of("/data");
    points.sort();
    assert_eq!(points, vec!["/old".to_string(), "/srv".to_string()]);
    // The leading slash is the fstab spelling and the mount table's is not — the same
    // subvolume either way, and a comparison that missed that would release nothing.
    assert_eq!(
        core.current_mounts_of("data"),
        core.current_mounts_of("/data")
    );
    assert!(core.current_mounts_of("/nowhere").is_empty());
}
