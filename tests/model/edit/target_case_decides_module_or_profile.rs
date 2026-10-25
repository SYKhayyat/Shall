use super::*;

#[test]
fn target_case_decides_module_or_profile() {
    let o = Origin::argument();
    assert_eq!(
        Target::parse("editors", &o).unwrap(),
        Target::Module(ModuleName::literal("editors"))
    );
    assert_eq!(
        Target::parse("Work", &o).unwrap(),
        Target::Profile("Work".into())
    );
    assert!(Target::parse("9lives", &o).is_err());
}
