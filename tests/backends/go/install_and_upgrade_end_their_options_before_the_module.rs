use super::*;

#[tokio::test]
async fn install_and_upgrade_end_their_options_before_the_module() {
    let vfs = Arc::new(dashmap::DashMap::new());
    let mock = Arc::new(shall::core::executor::MockExecutor::new(vfs.clone()));
    let exec = CommandExecutor::with_layer(
        false,
        false,
        mock.clone(),
        vfs,
        Arc::new(dashmap::DashMap::new()),
    );
    let core = Arc::new(GoBackendCore::new(exec));

    let spec = PackageSpec {
        name: "github.com/junegunn/fzf".into(),
        backend: "go".into(),
        ..Default::default()
    };
    GoInstallable { core: core.clone() }
        .install(&[spec], false)
        .await
        .unwrap();

    assert_eq!(
        mock.get_calls().await,
        vec!["go install -- github.com/junegunn/fzf@latest"]
    );
}
