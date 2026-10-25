use super::*;

#[tokio::test]
async fn no_priority_keeps_registration_order() {
    let providers = vec![fake("btrfs", true), fake("zfs", true)];
    let chosen = SnapshotManager::choose(providers, &[]).await.unwrap();
    assert_eq!(
        chosen.name(),
        "btrfs",
        "built-ins first when no list is declared"
    );
}
