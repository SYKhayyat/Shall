use super::*;

/// `info` is what the planner asks, so it has to answer the same way `list` does. It used
/// to answer `Path::exists`, which calls any directory on any filesystem a subvolume — and
/// a `sync` told the declaration is already satisfied never creates the subvolume at all.
#[tokio::test]
async fn a_plain_directory_is_not_an_installed_subvolume() {
    let core = core_with(
        "/dev/sdb1 /mnt/fs btrfs rw,subvol=/ 0 0\n\
         /dev/sdb1 /srv btrfs rw,subvol=/data 0 0\n",
        &[
            (
                "btrfs subvolume list /mnt/fs",
                "ID 256 gen 8 top level 5 path data\n",
            ),
            (
                "btrfs subvolume list /srv",
                "ID 256 gen 8 top level 5 path data\n",
            ),
        ],
    );
    let q = BtrfsQueryable {
        core: Arc::new(core),
    };
    // The temp directory exists on every machine that runs this suite, and it is not a
    // subvolume anywhere.
    let dir = std::env::temp_dir();
    assert!(dir.exists(), "fixture assumption");
    assert!(q.info(dir.to_str().unwrap()).await.unwrap().is_none());

    // And the real one answers WITH its properties, which is the half the planner reads.
    let found = q.info("/mnt/fs/data").await.unwrap().expect("declared");
    assert_eq!(
        found.properties.get("mount").map(String::as_str),
        Some("/srv")
    );
}
