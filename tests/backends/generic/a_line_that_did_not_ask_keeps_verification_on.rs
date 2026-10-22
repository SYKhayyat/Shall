use super::*;

/// The default is untouched: a line that did not ask keeps the manager's verification on.
#[tokio::test]
async fn a_line_that_did_not_ask_keeps_verification_on() {
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    let inst = GenericInstallable {
        core: Arc::new(source_option_core(mock.clone(), vfs)),
    };

    inst.install(
        &[spec_with(
            "diff",
            &[("url", "https://github.com/databus23/helm-diff")],
        )],
        false,
    )
    .await
    .unwrap();

    let calls = mock.get_calls().await;
    assert!(
        !calls.iter().any(|c| c.contains("--verify")),
        "verification must stay on unless the line said otherwise: {:?}",
        calls
    );
}
