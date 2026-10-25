use super::*;

/// `list_installed` asked `btrfs subvolume list -p /` until 2026-07-30 — one filesystem,
/// the one mounted at `/`, whatever the declaration said.
///
/// So `btrfs:/mnt/data/vol` was created by `install` and never seen by `list`, and on a
/// machine whose root is not btrfs the query failed outright. A name `list` does not return
/// is a package `sync` believes is absent: it re-creates it on every run, for ever. The
/// backend was excused from every harness as "a snapshot provider, not an install target",
/// which is why nothing ever noticed.
#[tokio::test]
async fn a_subvolume_is_listed_by_the_path_install_was_given() {
    let core = core_with(
        "/dev/sda1 / ext4 rw,relatime 0 0\n\
         /dev/sdb1 /mnt/data btrfs rw,relatime,subvol=/ 0 0\n",
        &[(
            "btrfs subvolume list /mnt/data",
            "ID 256 gen 8 top level 5 path vol\n",
        )],
    );
    let q = BtrfsQueryable {
        core: Arc::new(core),
    };
    let names: Vec<String> = q
        .list_installed()
        .await
        .unwrap()
        .into_iter()
        .map(|p| p.name)
        .collect();
    assert_eq!(
        names,
        vec!["/mnt/data/vol".to_string()],
        "`install` was handed /mnt/data/vol, so `list` has to say the same string"
    );
}
