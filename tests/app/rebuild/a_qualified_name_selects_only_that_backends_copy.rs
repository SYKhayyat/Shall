use super::*;

#[test]
fn a_qualified_name_selects_only_that_backends_copy() {
    let declared = vec![spec("cargo", "fd"), spec("apt", "fd")];
    let p = plan(
        &Scope::Packages(targets(&["cargo:fd"])),
        &declared,
        &all_installed,
        &["apt".into(), "cargo".into()],
        &foundation,
    );
    assert_eq!(p.batches.len(), 1);
    assert_eq!(p.batches[0].backend, "cargo");
}
