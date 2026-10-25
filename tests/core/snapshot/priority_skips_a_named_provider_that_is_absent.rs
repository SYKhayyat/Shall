use super::*;

#[tokio::test]
async fn priority_skips_a_named_provider_that_is_absent() {
    // The list names zfs first, but zfs is not on this machine — so the next available named
    // one wins, not "nothing".
    let providers = vec![fake("btrfs", true), fake("zfs", false)];
    let chosen = SnapshotManager::choose(providers, &["zfs".into(), "btrfs".into()])
        .await
        .unwrap();
    assert_eq!(chosen.name(), "btrfs");
}
