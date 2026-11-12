use super::*;
use shall::core::executor::DryRunOutput;

/// The default is untouched: a line that did not ask keeps the manager's verification on.
#[tokio::test]
async fn a_line_that_did_not_ask_keeps_verification_on() {
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));

    // Stated rather than fallen through (`#162`): what this test asks its manager, and the
    // answer it read from silence before. The assertions here are about the argv.
    mock.set_response(
        "helm plugin install -- https://github.com/databus23/helm-diff",
        Ok(DryRunOutput::default().into()),
    );
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
