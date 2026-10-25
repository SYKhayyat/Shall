use super::*;

#[test]
fn a_module_name_is_lowercased_once_at_construction() {
    assert_eq!(ModuleName::new("Editors").unwrap().as_str(), "editors");
}
