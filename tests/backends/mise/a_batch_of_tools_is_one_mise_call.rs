use super::*;
use shall::core::executor::DryRunOutput;

/// Q45: **one command for N packages, not N commands.**
///
/// The generic backend batches; this one is hand-written and did not. `mise` takes a
/// list, so N one at a time is N of whatever that command costs — and where it runs under
/// `run_exclusive`, N serialised lock acquisitions on top.
#[tokio::test]
async fn a_batch_of_tools_is_one_mise_call() {
    let vfs = Arc::new(dashmap::DashMap::new());
    let mock = Arc::new(shall::core::executor::MockExecutor::new(vfs.clone()));

    // Stated rather than fallen through (`#162`): what this test asks its manager, and the
    // answer it read from silence before. The assertions here are about the argv.
    mock.set_response(
        "mise uninstall -- node go",
        Ok(DryRunOutput::default().into()),
    );
    mock.set_response(
        "mise use -g -- node@latest go@latest",
        Ok(DryRunOutput::default().into()),
    );

    // Stated rather than fallen through (`#162`): what this test asks its manager, and the
    // answer it read from silence before. The assertions here are about the argv.
    mock.set_response(
        "mise uninstall -- node go",
        Ok(DryRunOutput::default().into()),
    );
    mock.set_response(
        "mise use -g -- node@latest go@latest",
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
    let specs = vec![
        shall::core::PackageSpec {
            name: "node".into(),
            backend: "mise".into(),
            ..Default::default()
        },
        shall::core::PackageSpec {
            name: "go".into(),
            backend: "mise".into(),
            ..Default::default()
        },
    ];
    MiseInstallable { core: core.clone() }
        .install(&specs, false)
        .await
        .unwrap();
    MiseInstallable { core: core.clone() }
        .remove(
            &["node".to_string(), "go".to_string()],
            false,
            crate::harness::reaped_for_a_test(shall::app::sync::guard::GuardScope::Remove).await,
        )
        .await
        .unwrap();

    let calls = mock.get_calls().await;
    assert_eq!(
        calls.len(),
        2,
        "expected 2 command(s) for the whole batch, got {}: {:?}",
        calls.len(),
        calls
    );
    assert!(
        calls[0].contains("node@latest") && calls[0].contains("go@latest"),
        "{:?}",
        calls
    );
    assert!(
        calls[1].contains("node") && calls[1].contains("go"),
        "{:?}",
        calls
    );
}
