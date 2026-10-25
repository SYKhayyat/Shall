use super::*;

/// Same path on two filesystems is two subvolumes, and collapsing them would hide one.
#[tokio::test]
async fn the_same_name_on_two_filesystems_is_two_subvolumes() {
    let core = core_with(
        "/dev/sdb1 /mnt/a btrfs rw,subvol=/ 0 0\n\
         /dev/sdc1 /mnt/b btrfs rw,subvol=/ 0 0\n",
        &[
            (
                "btrfs subvolume list /mnt/a",
                "ID 256 gen 8 top level 5 path data\n",
            ),
            (
                "btrfs subvolume list /mnt/b",
                "ID 256 gen 8 top level 5 path data\n",
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
    assert_eq!(names, vec!["/mnt/a/data", "/mnt/b/data"]);
}
