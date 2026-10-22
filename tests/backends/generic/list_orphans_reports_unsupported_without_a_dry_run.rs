use super::*;

#[tokio::test]
async fn list_orphans_reports_unsupported_without_a_dry_run() {
    // A generic backend with no `orphan_dry_run` cannot say what its orphan verb would
    // delete, so it reports Unsupported and never removes blind.
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    let core = Arc::new(apt_like_core(mock, vfs)); // apt_like_core sets orphan_dry_run: None
    let up = GenericUpgradable { core };
    match up.list_orphans().await {
        Err(shall::core::Error::Unsupported(name)) => assert_eq!(name, "apt"),
        other => panic!("expected Unsupported, got {:?}", other),
    }
}
