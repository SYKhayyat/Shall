use super::*;

/// A umount that fails must stop the removal, not step over it: btrfs cannot delete a
/// mounted subvolume, so swallowing the failure ran `delete` straight into EBUSY after the
/// fstab entry was already dropped — half-torn, discovered late.
#[tokio::test]
async fn a_subvolume_that_cannot_be_unmounted_is_not_deleted() {
    // `umount` answers with a failure — the manager's answer when something holds the
    // mount open.
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    // The mock matches the full command line, not the program name.
    mock.set_response("umount /srv", Err(Error::Io("target is busy".to_string())));
    // Not a dry run: a dry-run executor diverts mutations into its VFS and never asks
    // the mock, so the registered umount failure would be silently swallowed by the
    // harness instead of reaching the code under test.
    let exec = CommandExecutor::with_layer(false, false, mock, vfs, Arc::new(DashMap::new()));
    let stem = format!(
        "shall-btrfs-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    );
    let mounts = std::env::temp_dir().join(format!("{stem}-mounts"));
    std::fs::write(&mounts, "/dev/sdb1 /mnt/fs btrfs rw,subvol=/ 0 0\n")
        .expect("fixture mount table");
    let core = BtrfsBackendCore {
        executor: exec,
        name: "btrfs".to_string(),
        mounts_file: mounts,
        fstab_file: std::env::temp_dir().join(format!("{stem}-fstab")),
    };
    let inst = BtrfsInstallable {
        core: Arc::new(core),
    };
    std::fs::write(
        &inst.core.fstab_file,
        "UUID=abc /srv btrfs subvol=/data 0 0\n",
    )
    .expect("fixture fstab");

    let err = inst
        .remove(
            &["/mnt/fs/data".to_string()],
            false,
            crate::harness::reaped_for_a_test(shall::app::sync::guard::GuardScope::Sync).await,
        )
        .await
        .expect_err("a busy subvolume cannot be removed; refusing beats EBUSY later");

    assert!(
        err.to_string().contains("mounted"),
        "the refusal says what held the subvolume: {err}"
    );
}
