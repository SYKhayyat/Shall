use super::*;

#[test]
fn active_refuses_a_module_name() {
    // Only profiles can be activated (II.4).
    let err = parse_active(&PathBuf::from("active"), "editors\n", &facts()).unwrap_err();
    assert!(err.hint.unwrap().contains("Only profiles can be activated"));
}
