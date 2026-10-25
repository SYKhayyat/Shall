use super::*;

#[tokio::test]
async fn a_create_only_config_provider_refuses_restore_and_names_the_snapshot() {
    let p = ConfigSnapshotProvider {
        executor: CommandExecutor::new(true, false),
        def: def("lvm"),
    };
    let err = p
        .restore("shall_pre_sync_20260726_120000")
        .await
        .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("shall_pre_sync_20260726_120000"), "{}", msg);
    assert!(msg.contains("cannot roll"), "{}", msg);
}
