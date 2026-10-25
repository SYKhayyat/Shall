use super::*;

#[test]
fn a_protected_package_is_dropped_and_reported() {
    let mut p = plan(
        &Scope::All,
        &[spec("apt", "bash"), spec("apt", "curl")],
        &all_installed,
        &["apt".into()],
        &foundation,
    );
    without_protected(&mut p, &|_, name| {
        (name == "bash").then(|| "protected by config rule `bash`".to_string())
    });
    assert_eq!(p.batches[0].names(), vec!["curl"]);
    assert_eq!(p.skipped[0].key, "apt:bash");
}
