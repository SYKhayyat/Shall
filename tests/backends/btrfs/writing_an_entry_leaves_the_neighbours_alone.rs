use super::*;

/// The line this replaced dropped every fstab line *containing* the mount point, so
/// declaring `/mnt` deleted `/mnt/data`, `/mnt/home`, and any comment naming the path.
#[test]
fn writing_an_entry_leaves_the_neighbours_alone() {
    let fstab = "# /etc/fstab: keep /mnt/data safe\n\
                 UUID=aaa / ext4 defaults 0 1\n\
                 UUID=bbb /mnt/data btrfs subvol=/data,defaults 0 0\n\
                 UUID=ccc /mnt/datastore ext4 defaults 0 2\n";
    let out = fstab_with(fstab, "bbb", "/srv", "/mnt", "defaults");
    assert!(out.contains("# /etc/fstab: keep /mnt/data safe"));
    assert!(out.contains("UUID=ccc /mnt/datastore"));
    assert!(out.contains("UUID=bbb /mnt/data btrfs subvol=/data"));
    assert!(out.contains("UUID=bbb /mnt btrfs subvol=/srv,defaults 0 0"));
}
