use super::*;

#[test]
fn a_package_at_the_module_root_still_carries_its_version() {
    let (name, version) = parse_go_version_m(MODULE_ROOT).unwrap();
    assert_eq!(name, "golang.org/x/example/hello");
    assert_eq!(
        version.as_deref(),
        Some("v0.0.0-20250915201037-7f05d217867b")
    );
}
