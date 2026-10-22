use super::*;

/// Q5. `@unverified` is what turns off a verification the *manager* does, and it reaches
/// the command line as that manager's own flag.
#[tokio::test]
async fn unverified_becomes_the_managers_own_opt_out_flag() {
    if !this_hosts_helm_takes_the_opt_out() {
        eprintln!("this host's helm rejects --verify; the premise of this test is false here");
        return;
    }
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    let inst = GenericInstallable {
        core: Arc::new(source_option_core(mock.clone(), vfs)),
    };

    let url = "https://github.com/databus23/helm-diff";
    inst.install(
        &[spec_with("diff", &[("url", url), ("unverified", "true")])],
        false,
    )
    .await
    .unwrap();

    let calls = mock.get_calls().await;
    assert!(
        calls.iter().any(|c| c.contains("--verify=false")),
        "the opt-out must reach the command: {:?}",
        calls
    );
}
