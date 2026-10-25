use super::*;

/// Removal has to take the entry with it. An fstab line naming a subvolume that no longer
/// exists is not untidy — it is a boot that stops in the initramfs.
#[test]
fn removing_a_subvolume_takes_its_fstab_entry_and_names_the_mount() {
    let fstab = "UUID=aaa / ext4 defaults 0 1\n\
                 UUID=bbb /srv btrfs subvol=/data,defaults 0 0\n";
    let (out, point) = fstab_without(fstab, "/data");
    assert_eq!(
        point.as_deref(),
        Some("/srv"),
        "so the caller can unmount it"
    );
    assert_eq!(out, "UUID=aaa / ext4 defaults 0 1\n");

    // A subvolume that was never mounted has no entry, and that is not an error.
    let (unchanged, none) = fstab_without(fstab, "/nothing");
    assert_eq!(unchanged, fstab);
    assert_eq!(none, None);
}
