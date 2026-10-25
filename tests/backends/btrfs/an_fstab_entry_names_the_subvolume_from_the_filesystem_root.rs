use super::*;

#[test]
fn an_fstab_entry_names_the_subvolume_from_the_filesystem_root() {
    let plain = mounts("/dev/sdb1 /mnt/fs btrfs rw,subvol=/ 0 0\n");
    assert_eq!(subvol_of(&plain, "/mnt/fs/data").as_deref(), Some("/data"));
    assert_eq!(
        subvol_of(&plain, "/mnt/fs/a/b").as_deref(),
        Some("/a/b"),
        "a nested subvolume keeps every component"
    );

    // The filesystem the subvolume lives on comes back with it, because the UUID an fstab
    // entry needs can only be asked for there: `btrfs filesystem show /mnt/fs/data` answers
    // `not a valid btrfs filesystem` (measured 2026-07-31), and asking it at the declared
    // path is why `@mount=` failed on its first real run.
    assert_eq!(
        subvol_arg(&plain, "/mnt/fs/data")
            .map(|(_, fs)| fs)
            .as_deref(),
        Some("/mnt/fs")
    );

    // The ordinary Linux root: `/` mounted at `subvol=/@`. An entry built from the declared
    // path would say `subvol=/srv`, which is a different object from `/@/srv` — or nothing
    // at all, and a boot that stops on a mount that cannot be found.
    let offset = mounts("/dev/sda2 / btrfs rw,subvol=/@ 0 0\n");
    assert_eq!(subvol_of(&offset, "/srv").as_deref(), Some("/@/srv"));

    // The longest mount wins, not the first: `/` is a prefix of everything.
    let nested = mounts(
        "/dev/sda2 / btrfs rw,subvol=/@ 0 0\n\
         /dev/sdb1 /mnt/fs btrfs rw,subvol=/ 0 0\n",
    );
    assert_eq!(subvol_of(&nested, "/mnt/fs/data").as_deref(), Some("/data"));
}
