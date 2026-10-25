use super::*;

/// Two filesystems. Asking `/` answered about one of them and silently omitted the other.
#[tokio::test]
async fn every_mounted_btrfs_is_asked_not_just_the_root() {
    let core = core_with(
        "/dev/sda2 / btrfs rw,subvol=/ 0 0\n\
         /dev/sdb1 /mnt/tank btrfs rw,subvol=/ 0 0\n",
        &[
            (
                "btrfs subvolume list /",
                "ID 256 gen 8 top level 5 path a\n",
            ),
            (
                "btrfs subvolume list /mnt/tank",
                "ID 256 gen 8 top level 5 path b\n",
            ),
        ],
    );
    let q = BtrfsQueryable {
        core: Arc::new(core),
    };
    let mut names: Vec<String> = q
        .list_installed()
        .await
        .unwrap()
        .into_iter()
        .map(|p| p.name)
        .collect();
    names.sort();
    assert_eq!(names, vec!["/a".to_string(), "/mnt/tank/b".to_string()]);
}
