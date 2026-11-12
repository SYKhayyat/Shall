use super::*;
use shall::core::executor::DryRunOutput;

/// And with nobody opting in, the flag is simply absent — the default is the refusal.
#[tokio::test]
async fn pip_without_the_opt_in_sends_no_flag() {
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));

    // Stated rather than fallen through (`#162`): what this test asks its manager, and the
    // answer it read from silence before. The assertions here are about the argv.
    mock.set_response("pip install -- black", Ok(DryRunOutput::default().into()));
    let inst = GenericInstallable {
        core: Arc::new(pip_core(mock.clone(), vfs)),
    };
    inst.install(&[pip_spec("black", false)], false)
        .await
        .unwrap();
    assert!(
        mock.get_calls()
            .await
            .iter()
            .all(|c| !c.contains("--break-system-packages")),
        "nothing asked for it"
    );
}
