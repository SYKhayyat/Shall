use super::*;

#[test]
fn a_batch_holds_one_backend_and_the_batches_are_ordered() {
    let declared = vec![
        spec("cargo", "ripgrep"),
        spec("apt", "curl"),
        spec("cargo", "fd"),
    ];
    let p = plan(
        &Scope::All,
        &declared,
        &all_installed,
        &["apt".into(), "cargo".into()],
        &foundation,
    );
    assert_eq!(p.batches.len(), 2);
    assert_eq!(p.batches[0].backend, "apt");
    assert_eq!(p.batches[1].names(), vec!["fd", "ripgrep"]);
}
