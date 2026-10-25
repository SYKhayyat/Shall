use super::*;

/// **NotFound is the only read error that means empty.** An fstab that exists but cannot
/// be read is not an empty one: the old `unwrap_or_default()` made `update_fstab` write a
/// file containing only the new entry — every other entry gone — and made
/// `drop_from_fstab` say "nothing to drop" so the caller deleted a subvolume whose fstab
/// entry survived it. The fixture points `fstab_file` at a directory, which no platform
/// will read as a file.
#[test]
fn an_unreadable_fstab_is_refused_not_treated_as_empty() {
    let core = core_with("/dev/sdb1 /mnt/fs btrfs rw,subvol=/ 0 0\n", &[]);
    let _ = std::fs::remove_file(&core.fstab_file);
    std::fs::create_dir_all(&core.fstab_file).expect("fixture: a directory as fstab");

    let e = core
        .update_fstab("abc", "/data", "/srv", "defaults")
        .expect_err(
            "writing into an unreadable fstab would destroy entries this process cannot see",
        );
    assert!(e.to_string().contains("unreadable"), "{e}");

    let e = core
        .drop_from_fstab("/data")
        .expect_err("dropping from an unreadable fstab reports nothing and lies by omission");
    assert!(e.to_string().contains("unreadable"), "{e}");
}
