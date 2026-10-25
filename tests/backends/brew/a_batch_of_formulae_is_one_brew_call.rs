use super::*;

/// Q45: **one command for N packages, not N commands.**
///
/// The generic backend batches; this one is hand-written and did not. `brew` takes a
/// list, so N one at a time is N of whatever that command costs — and where it runs under
/// `run_exclusive`, N serialised lock acquisitions on top.
#[tokio::test]
async fn a_batch_of_formulae_is_one_brew_call() {
    let vfs = Arc::new(dashmap::DashMap::new());
    let mock = Arc::new(shall::core::executor::MockExecutor::new(vfs.clone()));
    let exec = CommandExecutor::with_layer(
        false,
        false,
        mock.clone(),
        vfs,
        Arc::new(dashmap::DashMap::new()),
    );
    let core = Arc::new(BrewBackendCore::new(exec));
    let specs = vec![
        shall::core::PackageSpec {
            name: "jq".into(),
            backend: "brew".into(),
            ..Default::default()
        },
        shall::core::PackageSpec {
            name: "ripgrep".into(),
            backend: "brew".into(),
            ..Default::default()
        },
    ];
    BrewInstallable { core: core.clone() }
        .install(&specs, false)
        .await
        .unwrap();
    BrewInstallable { core: core.clone() }
        .remove(
            &["jq".to_string(), "ripgrep".to_string()],
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
        calls[0].contains("jq") && calls[0].contains("ripgrep"),
        "{:?}",
        calls
    );
    assert!(
        calls[1].contains("jq") && calls[1].contains("ripgrep"),
        "{:?}",
        calls
    );
}
