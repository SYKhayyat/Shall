use super::*;

/// `info` is what the planner asks, so it is what has to carry the geometry. Both reads are
/// per-subvolume and live here rather than in `list_installed`, which walks every subvolume
/// on every filesystem — asking there would spawn a subprocess per subvolume on every sync.
#[tokio::test]
async fn info_reports_the_quota_and_the_mount_options_a_declaration_can_drift_from() {
    let core = core_with(
        "/dev/sdb1 /mnt/fs btrfs rw,relatime,subvol=/ 0 0\n",
        &[
            (
                "btrfs subvolume list /mnt/fs",
                "ID 256 gen 8 top level 5 path data\n",
            ),
            (
                "btrfs qgroup show -r -f --raw /mnt/fs/data",
                "qgroupid         rfer         excl     max_rfer \n\
                  --------         ----         ----     -------- \n\
                  0/256            16384        16384    10737418240\n",
            ),
        ],
    );
    std::fs::write(
        &core.fstab_file,
        "UUID=abc /srv btrfs subvol=/data,noatime 0 0\n",
    )
    .expect("fixture fstab");
    let q = BtrfsQueryable {
        core: Arc::new(core),
    };
    let p = q
        .info("/mnt/fs/data")
        .await
        .unwrap()
        .expect("the subvolume is listed");
    assert_eq!(
        p.properties.get("quota").map(String::as_str),
        Some("10737418240")
    );
    assert_eq!(
        p.properties.get("mount_options").map(String::as_str),
        Some("noatime")
    );
}
