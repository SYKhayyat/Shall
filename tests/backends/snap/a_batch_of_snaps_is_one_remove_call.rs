use super::*;

/// Q45. Removal batches; install does not, and the asymmetry is the point — `snap install`
/// has to choose per package between `install` and `refresh --channel=` depending on what
/// is already there (D13, Q20), so those specs cannot share a command. Removal asks no
/// such question.
#[tokio::test]
async fn a_batch_of_snaps_is_one_remove_call() {
    let vfs = Arc::new(dashmap::DashMap::new());
    let mock = Arc::new(shall::core::executor::MockExecutor::new(vfs.clone()));
    let exec = CommandExecutor::with_layer(
        false,
        false,
        mock.clone(),
        vfs,
        Arc::new(dashmap::DashMap::new()),
    );
    let core = Arc::new(SnapBackendCore::new(exec));

    SnapInstallable { core: core.clone() }
        .remove(
            &["code".to_string(), "firefox".to_string()],
            false,
            crate::harness::reaped_for_a_test(shall::app::sync::guard::GuardScope::Remove).await,
        )
        .await
        .unwrap();

    let calls = mock.get_calls().await;
    assert_eq!(calls.len(), 1, "one removal for the batch, got {:?}", calls);
    assert_eq!(calls[0], "snap remove -- code firefox");
}
