use super::*;
use shall::core::executor::DryRunOutput;

#[tokio::test]
async fn mise_tool_names_come_after_the_terminator() {
    let vfs = Arc::new(dashmap::DashMap::new());
    let mock = Arc::new(shall::core::executor::MockExecutor::new(vfs.clone()));

    // Stated rather than fallen through (`#162`): what this test asks its manager, and the
    // answer it read from silence before. The assertions here are about the argv.
    mock.set_response("mise uninstall -- node", Ok(DryRunOutput::default().into()));
    mock.set_response(
        "mise use -g -- node@latest",
        Ok(DryRunOutput::default().into()),
    );

    // Stated rather than fallen through (`#162`): what this test asks its manager, and the
    // answer it read from silence before. The assertions here are about the argv.
    mock.set_response("mise uninstall -- node", Ok(DryRunOutput::default().into()));
    mock.set_response(
        "mise use -g -- node@latest",
        Ok(DryRunOutput::default().into()),
    );
    let exec = CommandExecutor::with_layer(
        false,
        false,
        mock.clone(),
        vfs,
        Arc::new(dashmap::DashMap::new()),
    );
    let core = Arc::new(MiseBackendCore::new(exec));

    MiseInstallable { core: core.clone() }
        .install(
            &[PackageSpec {
                name: "node".into(),
                backend: "mise".into(),
                ..Default::default()
            }],
            false,
        )
        .await
        .unwrap();
    MiseInstallable { core: core.clone() }
        .remove(
            &["node".to_string()],
            false,
            crate::harness::reaped_for_a_test(shall::app::sync::guard::GuardScope::Remove).await,
        )
        .await
        .unwrap();

    assert_eq!(
        mock.get_calls().await,
        vec!["mise use -g -- node@latest", "mise uninstall -- node"]
    );
}
