use super::*;

#[tokio::test]
async fn priority_picks_the_first_available_in_the_declared_order() {
    // btrfs and zfs both present; the list prefers zfs, so zfs wins over registration order.
    let providers = vec![fake("btrfs", true), fake("zfs", true)];
    let chosen = SnapshotManager::choose(providers, &["zfs".into(), "btrfs".into()])
        .await
        .unwrap();
    assert_eq!(chosen.name(), "zfs");
}
