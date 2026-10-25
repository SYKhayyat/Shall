use super::*;

/// The collapsed second name is not thrown away — it is *where the subvolume is mounted*,
/// and `sync` needs it to tell a `@mount=` that took effect from one that did not.
#[tokio::test]
async fn a_mounted_subvolume_reports_where_it_is_mounted() {
    let core = core_with(
        "/dev/sdb1 /mnt/fs btrfs rw,subvol=/ 0 0\n\
         /dev/sdb1 /srv btrfs rw,subvol=/data 0 0\n",
        &[
            (
                "btrfs subvolume list /mnt/fs",
                "ID 256 gen 8 top level 5 path data\n\
                 ID 257 gen 8 top level 5 path other\n",
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
    let pkgs = q.list_installed().await.unwrap();
    let data = pkgs.iter().find(|p| p.name == "/mnt/fs/data").unwrap();
    assert_eq!(
        data.properties.get("mount").map(String::as_str),
        Some("/srv")
    );
    // A subvolume nobody mounted reports no mountpoint, and that is a *fact* — the mount
    // table answered and this subvolume is in none of it. The planner reads it as "not
    // where the line says", which is what makes a `@mount=` that failed converge on the
    // next run instead of being invisible for ever.
    let other = pkgs.iter().find(|p| p.name == "/mnt/fs/other").unwrap();
    assert_eq!(other.properties.get("mount"), None);
}
