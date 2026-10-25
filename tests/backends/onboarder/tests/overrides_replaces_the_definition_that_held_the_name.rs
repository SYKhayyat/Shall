use super::*;

/// Q6. `overrides = true` replaces whatever already holds the name. Here that is an
/// earlier custom definition; `a_definition_that_says_so_replaces_a_built_in` in
/// `registry.rs` covers the case the key exists for.
#[tokio::test]
async fn overrides_replaces_the_definition_that_held_the_name() {
    let (mock, exec) = mock_exec();
    let mut reg = BackendRegistry::new();

    let first = CustomBackendDef {
        name: "paru".into(),
        install_args: vec!["-S".into()],
        list_args: vec!["-Qm".into()],
        ..Default::default()
    };
    let second = CustomBackendDef {
        name: "paru".into(),
        binary: Some("paru-git".into()),
        install_args: vec!["-S".into(), "--noconfirm".into()],
        list_args: vec!["-Qm".into()],
        overrides: true,
        ..Default::default()
    };
    assert_eq!(
        register_custom_backends(&mut reg, &exec, vec![first, second]),
        2,
        "both definitions were accepted — the second replacing the first"
    );

    reg.get("paru")
        .unwrap()
        .as_installable()
        .unwrap()
        .install(&[spec("paru", "jq")], false)
        .await
        .unwrap();
    let calls = mock.get_calls().await;
    assert!(
        calls.iter().any(|c| c.starts_with("paru-git -S")),
        "the second definition did not win: {:?}",
        calls
    );
}
