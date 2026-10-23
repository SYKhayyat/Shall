use super::*;

/// The same half-state by the other route — a directory with no installed manifest.
#[test]
fn an_app_with_no_version_is_not_an_installed_package() {
    assert!(!parse_scoop_export(EXPORT)
        .expect("this fixture parses")
        .iter()
        .any(|p| p.name == "halfway"));
}
