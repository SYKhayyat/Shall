use super::*;

/// One declaration, one entry — whichever half changed.
#[test]
fn re_declaring_replaces_rather_than_doubles() {
    let first = fstab_with(
        "UUID=aaa / ext4 defaults 0 1\n",
        "bbb",
        "/srv",
        "/mnt/x",
        "defaults",
    );
    // The same subvolume at a new mount point leaves nothing behind at the old one.
    let moved = fstab_with(&first, "bbb", "/srv", "/mnt/y", "defaults");
    assert_eq!(moved.matches("subvol=/srv").count(), 1);
    assert!(!moved.contains("/mnt/x"));
    // A different subvolume at an occupied mount point replaces the tenant.
    let swapped = fstab_with(&moved, "bbb", "/other", "/mnt/y", "noatime");
    assert_eq!(swapped.matches(" /mnt/y ").count(), 1);
    assert!(!swapped.contains("subvol=/srv"));
    assert!(swapped.contains("subvol=/other,noatime"));
    assert!(swapped.contains("UUID=aaa / ext4"), "the root is untouched");
}
