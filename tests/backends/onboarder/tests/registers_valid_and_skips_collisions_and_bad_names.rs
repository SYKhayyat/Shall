use super::*;

#[test]
fn registers_valid_and_skips_collisions_and_bad_names() {
    let exec = CommandExecutor::new(true, false);
    let mut reg = BackendRegistry::new();

    let good = CustomBackendDef {
        name: "paru".into(),
        install_args: vec!["-S".into()],
        remove_args: vec!["-R".into()],
        list_args: vec!["-Qm".into()],
        ..Default::default()
    };
    let bad_name = CustomBackendDef {
        name: "bad name/x".into(),
        ..good.clone()
    };
    let collision = CustomBackendDef {
        name: "paru".into(),
        ..good.clone()
    };

    let n = register_custom_backends(&mut reg, &exec, vec![good, bad_name, collision]);
    assert_eq!(
        n, 1,
        "only the first valid, non-colliding backend registers"
    );

    let caps = reg.get("paru").expect("paru registered");
    assert!(caps.is_installable());
    assert!(caps.is_queryable());
    // no search_args â‡’ not searchable; no upgrade_args â‡’ not upgradable
    assert!(!caps.is_searchable());
    assert!(!caps.is_upgradable());
    assert!(caps.is_metadata_provider());
}
