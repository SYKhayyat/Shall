use super::*;

#[test]
fn declared_but_not_installed_is_left_to_sync() {
    let p = plan(
        &Scope::All,
        &[spec("apt", "curl")],
        &|_, name| name != "curl",
        &["apt".into()],
        &foundation,
    );
    assert!(p.is_empty());
    assert_eq!(p.skipped[0].key, "apt:curl");
    assert!(p.skipped[0].reason.contains("sync"));
}
