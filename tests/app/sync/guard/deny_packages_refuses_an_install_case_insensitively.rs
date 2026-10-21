use super::*;

#[test]
fn deny_packages_refuses_an_install_case_insensitively() {
    let guard = shall::config::GuardSettings {
        deny_packages: vec!["LeftPad".into()],
        ..Default::default()
    };
    let os = inspect_desired(&guard, &desired(&[("npm", "leftpad", None)]));
    assert!(
        matches!(os.as_slice(), [Objection::Denied { .. }]),
        "{:?}",
        os
    );
}
