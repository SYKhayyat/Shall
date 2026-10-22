use super::*;

/// And with nobody opting in, the flag is simply absent — the default is the refusal.
#[tokio::test]
async fn pip_without_the_opt_in_sends_no_flag() {
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
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
