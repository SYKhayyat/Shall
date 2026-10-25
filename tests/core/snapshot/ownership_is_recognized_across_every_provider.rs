use super::*;

#[test]
fn ownership_is_recognized_across_every_provider() {
    // S3: the marker lands in different fields per provider. All of these are Shall's.
    assert!(snap(
        "shall_pre_pre_sync_20260717143022",
        "BTRFS System State",
        "btrfs"
    )
    .is_shall_owned());
    assert!(snap("tank/root@shall_20260717_143022", "ZFS Snapshot", "zfs").is_shall_owned());
    // Windows: id is a bare sequence number, marker is in the description — the case the
    // old id-only filter missed entirely.
    assert!(snap("12", "Shall: pre_sync", "windows_restore").is_shall_owned());
}
