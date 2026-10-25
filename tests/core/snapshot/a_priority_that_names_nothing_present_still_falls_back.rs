use super::*;

#[tokio::test]
async fn a_priority_that_names_nothing_present_still_falls_back() {
    let providers = vec![fake("btrfs", true)];
    let chosen = SnapshotManager::choose(providers, &["apfs".into()])
        .await
        .unwrap();
    assert_eq!(chosen.name(), "btrfs");
}
