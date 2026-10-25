use super::*;

#[test]
fn a_package_line_has_no_extra_key() {
    // Packages are tracked by the registry, not this ledger.
    assert!(extra_key(&Statement::Subtract("vim".into()))
        .unwrap()
        .is_none());
}
