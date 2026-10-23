use super::*;

/// The scar. A failed install stays in scoop's listing forever and is not installed
/// software; `adopt` wrote one into a manifest once.
#[test]
fn a_failed_install_is_not_an_installed_package() {
    assert!(!parse_scoop_export(EXPORT)
        .expect("this fixture parses")
        .iter()
        .any(|p| p.name == "jq"));
}
