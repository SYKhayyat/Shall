use super::*;

/// The `subvol=` offset, which is the normal layout on a btrfs root (openSUSE, Fedora and
/// Garuda all mount `/` at `subvol=/@`). Without it every name is wrong by one component.
#[tokio::test]
async fn a_mount_at_a_subvolume_reports_paths_relative_to_that_mount() {
    let core = core_with(
        "/dev/sda2 / btrfs rw,relatime,subvol=/@ 0 0\n",
        &[(
            "btrfs subvolume list /",
            "ID 256 gen 8 top level 5 path @\n\
             ID 257 gen 9 top level 256 path @/srv\n\
             ID 258 gen 9 top level 5 path @home\n",
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
    // `@` is the mount itself; `@/srv` is under it; `@home` is a sibling subvolume with no
    // path through this mount and must not be named, because no verb could act on it.
    assert_eq!(names, vec!["/".to_string(), "/srv".to_string()]);
}
