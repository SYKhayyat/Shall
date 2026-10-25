use super::*;

#[test]
fn backend_scope_takes_that_backend_whole() {
    let declared = vec![
        spec("cargo", "fd"),
        spec("apt", "curl"),
        spec("cargo", "rg"),
    ];
    let p = plan(
        &Scope::Backend("cargo".into()),
        &declared,
        &all_installed,
        &["apt".into(), "cargo".into()],
        &foundation,
    );
    assert_eq!(p.batches.len(), 1);
    assert_eq!(p.batches[0].names(), vec!["fd", "rg"]);
}
