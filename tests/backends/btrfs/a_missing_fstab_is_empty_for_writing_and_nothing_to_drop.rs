use super::*;

#[test]
fn a_missing_fstab_is_empty_for_writing_and_nothing_to_drop() {
    // The control that keeps the refusal above honest: genuinely absent means the first
    // entry may be written, and there is nothing to drop.
    let core = core_with("/dev/sdb1 /mnt/fs btrfs rw,subvol=/ 0 0\n", &[]);
    let _ = std::fs::remove_file(&core.fstab_file);
    core.update_fstab("abc", "/data", "/srv", "defaults")
        .expect("no fstab yet means the declaration writes its first entry");
    assert!(core.fstab_file.exists());
    let dropped = core.drop_from_fstab("/data").expect("readable now");
    assert_eq!(dropped.as_deref(), Some("/srv"));
}
