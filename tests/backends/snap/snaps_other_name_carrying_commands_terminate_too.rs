use super::*;
use shall::core::executor::DryRunOutput;

#[tokio::test]
async fn snaps_other_name_carrying_commands_terminate_too() {
    let vfs = Arc::new(dashmap::DashMap::new());
    let mock = Arc::new(shall::core::executor::MockExecutor::new(vfs.clone()));

    // Stated rather than fallen through (`#162`): what this test asks its manager, and the
    // answer it read from silence before. The assertions here are about the argv.
    mock.set_response("snap find -- code", Ok(DryRunOutput::default().into()));
    mock.set_response("snap info -- code", Ok(DryRunOutput::default().into()));
    mock.set_response("snap remove -- code", Ok(DryRunOutput::default().into()));

    // Stated rather than fallen through (`#162`): what this test asks its manager, and the
    // answer it read from silence before. The assertions here are about the argv.
    mock.set_response("snap find -- code", Ok(DryRunOutput::default().into()));
    mock.set_response("snap info -- code", Ok(DryRunOutput::default().into()));
    mock.set_response("snap remove -- code", Ok(DryRunOutput::default().into()));
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
            &["code".to_string()],
            false,
            crate::harness::reaped_for_a_test(shall::app::sync::guard::GuardScope::Remove).await,
        )
        .await
        .unwrap();
    SnapQueryable { core: core.clone() }.info("code").await.ok();
    SnapSearchable { core: core.clone() }
        .search("code")
        .await
        .unwrap();
    core.get_dependencies("code").await.unwrap();

    assert_eq!(
        mock.get_calls().await,
        vec![
            "snap remove -- code",
            "snap info -- code",
            "snap find -- code",
            "snap info -- code",
        ]
    );
}
