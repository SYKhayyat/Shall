use super::*;

/// Q6, the case the key exists for: a manager changes its CLI, and the person on that
/// machine corrects it that day instead of waiting for a Shall release.
#[tokio::test]
async fn a_definition_that_says_so_replaces_a_built_in() {
    use dashmap::DashMap;
    use shall::backends::onboarder::{register_custom_backends, CustomBackendDef};
    use shall::core::executor::MockExecutor;
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    let exec =
        CommandExecutor::with_layer(true, false, mock.clone(), vfs, Arc::new(DashMap::new()));
    let mut reg = BackendRegistry::new();
    register_apt(&mut reg, &exec);

    let mine = CustomBackendDef {
        name: "apt".into(),
        binary: Some("apt-fast".into()),
        install_args: vec!["install".into(), "--assume-yes".into()],
        remove_args: vec!["remove".into()],
        list_args: vec!["list".into()],
        overrides: true,
        ..Default::default()
    };
    assert_eq!(register_custom_backends(&mut reg, &exec, vec![mine]), 1);

    reg.get("apt")
        .expect("apt is still registered")
        .as_installable()
        .expect("installs")
        .install(
            &[shall::core::PackageSpec {
                name: "jq".into(),
                backend: "apt".into(),
                ..Default::default()
            }],
            false,
        )
        .await
        .unwrap();

    let calls = mock.get_calls().await;
    assert!(
        calls.iter().any(|c| c.starts_with("apt-fast install")),
        "the user's definition did not win: {:?}",
        calls
    );
    assert!(
        !calls.iter().any(|c| c.starts_with("apt-get ")),
        "the built-in was still driving the install: {:?}",
        calls
    );
}
