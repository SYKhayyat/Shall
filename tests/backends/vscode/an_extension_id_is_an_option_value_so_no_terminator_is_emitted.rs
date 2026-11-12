use super::*;
use shall::core::executor::DryRunOutput;

#[tokio::test]
async fn an_extension_id_is_an_option_value_so_no_terminator_is_emitted() {
    let vfs = Arc::new(dashmap::DashMap::new());
    let mock = Arc::new(shall::core::executor::MockExecutor::new(vfs.clone()));

    // Stated rather than fallen through (`#162`): what this test asks its manager, and the
    // answer it read from silence before. The assertions here are about the argv.
    mock.set_response(
        "code --force --install-extension ms-python.python",
        Ok(DryRunOutput::default().into()),
    );
    mock.set_response(
        "code --uninstall-extension ms-python.python",
        Ok(DryRunOutput::default().into()),
    );

    // Stated rather than fallen through (`#162`): what this test asks its manager, and the
    // answer it read from silence before. The assertions here are about the argv.
    mock.set_response(
        "code --force --install-extension ms-python.python",
        Ok(DryRunOutput::default().into()),
    );
    mock.set_response(
        "code --uninstall-extension ms-python.python",
        Ok(DryRunOutput::default().into()),
    );
    let exec = CommandExecutor::with_layer(
        false,
        false,
        mock.clone(),
        vfs,
        Arc::new(dashmap::DashMap::new()),
    );
    let core = Arc::new(VscodeBackendCore::new(exec));

    VscodeInstallable { core: core.clone() }
        .install(
            &[PackageSpec {
                name: "ms-python.python".into(),
                backend: "vscode".into(),
                ..Default::default()
            }],
            false,
        )
        .await
        .unwrap();
    VscodeInstallable { core: core.clone() }
        .remove(
            &["ms-python.python".to_string()],
            false,
            crate::harness::reaped_for_a_test(shall::app::sync::guard::GuardScope::Remove).await,
        )
        .await
        .unwrap();

    assert_eq!(
        mock.get_calls().await,
        vec![
            "code --force --install-extension ms-python.python",
            "code --uninstall-extension ms-python.python",
        ]
    );
    assert!(!shall::core::argv::terminates_options("code"));
}
