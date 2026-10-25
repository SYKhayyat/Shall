use super::*;

/// `@mount` gives one subvolume a second path, and the second one is undeclared —
/// `remove-orphans` would offer to destroy the volume the user had just declared.
///
/// The surviving name must be the declared one and not merely the shorter one. A mount at
/// `/srv` is shorter than `/mnt/fs/data`, and answering `/srv` would leave the declaration
/// looking unfulfilled: `sync` would create the subvolume again on every run, which is the
/// bug this backend was fixed for on 2026-07-30, arriving from the other direction.
#[tokio::test]
async fn a_subvolume_reachable_twice_is_listed_once_by_its_declared_name() {
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
    let names: Vec<String> = q
        .list_installed()
        .await
        .unwrap()
        .into_iter()
        .map(|p| p.name)
        .collect();
    assert_eq!(
        names,
        vec!["/mnt/fs/data".to_string()],
        "one object, one package, named the way the declaration named it"
    );
}
