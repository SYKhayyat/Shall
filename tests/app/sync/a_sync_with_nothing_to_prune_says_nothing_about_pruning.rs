use super::*;
use crate::mock_providers::MockSnapshotProvider;
use shall::core::executor::DryRunOutput;
use shall::core::{RetentionPolicy, SnapshotManager};

/// The prune's report is guarded by `Ok(r) if !r.is_empty()`, and the guard is the whole of
/// what the mutation job could see: `prune_by_policy` already deleted the snapshots before the
/// match, so changing the guard changes only what is said. This test and the one for a prune
/// that *did* delete something are the pair — one pins "it says so when it does", this one
/// "it does not say a zero".
#[tokio::test]
async fn a_sync_with_nothing_to_prune_says_nothing_about_pruning() {
    let mut config = Config::sandboxed(&config_dir("prune-nothing"));
    config.retention.snapshots = RetentionPolicy {
        keep_last: 1,
        keep_days: 30,
        keep: Vec::new(),
    };
    let (mut engine, mock) = engine(config).await;

    // Stated rather than fallen through (`#162`): what this test asks its manager, and the
    // answer it read from silence before. The assertions here are about the argv.
    mock.set_response(
        "brew info --json=v1 -- neovim",
        Ok(DryRunOutput::default().into()),
    );

    let provider = MockSnapshotProvider::new();
    provider
        .add_historical_snapshot("shall_pre_recent", 1)
        .await;
    engine.snapshot_manager = Arc::new(SnapshotManager::with_provider(Box::new(provider)));
    mock.set_response(
        "brew info --json=v1 -- neovim",
        Ok(DryRunOutput::default().into()),
    );
    mock.set_response("brew install -- neovim", Ok(DryRunOutput::default().into()));
    let (logs, guard) = log_collector();

    engine
        .sync(
            install_changes(&[package("brew", "neovim")]),
            GuardScope::Sync,
        )
        .await
        .expect("the sync failed");
    drop(guard);

    let logs = logs.lock().expect("the log mutex").clone();
    assert!(
        !logs
            .iter()
            .any(|line| line.contains("pruned") && line.contains("snapshot")),
        "a prune that deleted nothing reported a prune: {logs:?}"
    );
}
