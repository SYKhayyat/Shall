use super::*;

/// Q45, and the one whose argv is not a name list.
///
/// **The flag repeats; the operand does not.** VS Code takes several extensions as
/// `--install-extension a --install-extension b`, so this cannot go through the generic
/// name-appending batcher — and each `code` launch starts an Electron process and rescans
/// the extension host, which is what N launches costs.
#[tokio::test]
async fn a_batch_of_extensions_is_one_code_process() {
    let vfs = Arc::new(dashmap::DashMap::new());
    let mock = Arc::new(shall::core::executor::MockExecutor::new(vfs.clone()));
    let exec = CommandExecutor::with_layer(
        false,
        false,
        mock.clone(),
        vfs,
        Arc::new(dashmap::DashMap::new()),
    );
    let core = Arc::new(VscodeBackendCore::new(exec));
    let specs = vec![
        shall::core::PackageSpec {
            name: "rust-lang.rust-analyzer".into(),
            backend: "vscode".into(),
            ..Default::default()
        },
        shall::core::PackageSpec {
            name: "tamasfe.even-better-toml".into(),
            backend: "vscode".into(),
            ..Default::default()
        },
    ];
    VscodeInstallable { core: core.clone() }
        .install(&specs, false)
        .await
        .unwrap();
    VscodeInstallable { core: core.clone() }
        .remove(
            &[
                "rust-lang.rust-analyzer".to_string(),
                "tamasfe.even-better-toml".to_string(),
            ],
            false,
            crate::harness::reaped_for_a_test(shall::app::sync::guard::GuardScope::Remove).await,
        )
        .await
        .unwrap();

    let calls = mock.get_calls().await;
    assert_eq!(calls.len(), 2, "one process each way, got {:?}", calls);
    assert_eq!(
        calls[0].matches("--install-extension").count(),
        2,
        "the flag has to repeat per extension: {:?}",
        calls
    );
    assert!(calls[0].contains("rust-lang.rust-analyzer"), "{:?}", calls);
    assert!(calls[0].contains("tamasfe.even-better-toml"), "{:?}", calls);
    assert_eq!(
        calls[1].matches("--uninstall-extension").count(),
        2,
        "{:?}",
        calls
    );
}
