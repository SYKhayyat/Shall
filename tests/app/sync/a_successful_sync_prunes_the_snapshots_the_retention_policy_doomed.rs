use super::*;
use crate::mock_providers::MockSnapshotProvider;
use shall::core::executor::DryRunOutput;
use shall::core::{RetentionPolicy, SnapshotManager};

#[tokio::test]
async fn a_successful_sync_prunes_the_snapshots_the_retention_policy_doomed() {
    let mut config = Config::sandboxed(&config_dir("prune-snapshots"));
    config.retention.snapshots = RetentionPolicy {
        keep_last: 1,
        keep_days: 1,
        keep: Vec::new(),
    };
    let (mut engine, mock) = engine(config).await;

    let provider = MockSnapshotProvider::new();
    let deletions = provider.deletions.clone();
    provider
        .add_historical_snapshot("shall_pre_recent", 1)
        .await;
    provider
        .add_historical_snapshot("shall_pre_ancient", 45)
        .await;
    engine.snapshot_manager = Arc::new(SnapshotManager::with_provider(Box::new(provider)));

    mock.set_response("brew install -- neovim", Ok(DryRunOutput::default().into()));
    let changes = install_changes(&[package("brew", "neovim")]);
    let (logs, guard) = log_collector();

    engine
        .sync(changes, GuardScope::Sync)
        .await
        .expect("the sync failed");
    drop(guard);

    let logs = logs.lock().expect("the log mutex").clone();
    assert!(
        logs.iter()
            .any(|line| line.contains("pruned 1 snapshot(s)")),
        "a prune that deleted a snapshot did not report it: {logs:?}"
    );
    let deleted = deletions.lock().await.clone();
    assert!(
        deleted.contains(&"shall_pre_ancient".to_string()),
        "the expired snapshot was not pruned: {deleted:?}"
    );
    assert!(
        !deleted.contains(&"shall_pre_recent".to_string()),
        "the newest snapshot was pruned: {deleted:?}"
    );
}
