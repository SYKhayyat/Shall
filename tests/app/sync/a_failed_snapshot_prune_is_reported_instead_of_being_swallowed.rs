use super::*;
use async_trait::async_trait;
use shall::core::executor::DryRunOutput;
use shall::core::snapshot::{RestoreCapability, SnapshotLabel};
use shall::core::{Error, Result, Snapshot, SnapshotManager};

struct FailingListProvider;

#[async_trait]
impl shall::core::SnapshotProvider for FailingListProvider {
    fn name(&self) -> &str {
        "failing-list"
    }

    async fn is_available(&self) -> bool {
        true
    }

    async fn create(&self, label: SnapshotLabel) -> Result<Snapshot> {
        Ok(Snapshot {
            id: "shall_pre_sync".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            description: label.to_string(),
            backend: "failing-list".to_string(),
        })
    }

    async fn list(&self) -> Result<Vec<Snapshot>> {
        Err(Error::Snapshot(
            "the provider could not list snapshots".to_string(),
        ))
    }

    async fn delete(&self, _id: &str) -> Result<()> {
        Ok(())
    }

    fn restore_capability(&self) -> RestoreCapability {
        RestoreCapability::Live
    }

    async fn restore(&self, _id: &str) -> Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn a_failed_snapshot_prune_is_reported_instead_of_being_swallowed() {
    let (mut engine, mock) = engine(Config::sandboxed(&config_dir("prune-failure"))).await;
    engine.snapshot_manager = Arc::new(SnapshotManager::with_provider(Box::new(
        FailingListProvider,
    )));
    mock.set_response("brew install -- neovim", Ok(DryRunOutput::default().into()));
    let changes = install_changes(&[package("brew", "neovim")]);
    let (logs, guard) = log_collector();

    let result = engine.sync(changes, GuardScope::Sync).await;
    drop(guard);

    result.expect("a failed retention prune must not fail the successful sync");
    let logs = logs.lock().expect("the log mutex").clone();
    assert!(
        logs.iter()
            .any(|line| line.contains("snapshot retention prune failed")),
        "the failed prune was not reported: {logs:?}"
    );
}
