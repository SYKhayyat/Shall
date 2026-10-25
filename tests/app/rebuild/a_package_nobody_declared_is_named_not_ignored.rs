use super::*;

#[test]
fn a_package_nobody_declared_is_named_not_ignored() {
    let p = plan(
        &Scope::Packages(targets(&["nosuch"])),
        &[spec("apt", "curl")],
        &all_installed,
        &["apt".into()],
        &foundation,
    );
    assert!(p.is_empty());
    assert_eq!(p.skipped[0].key, "nosuch");
    assert!(p.skipped[0].reason.contains("not declared"));
}
