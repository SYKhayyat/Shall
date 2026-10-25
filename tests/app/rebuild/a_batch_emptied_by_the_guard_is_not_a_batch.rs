use super::*;

#[test]
fn a_batch_emptied_by_the_guard_is_not_a_batch() {
    // An empty batch would print "rebuilding apt (0 packages)" and run two no-op syncs.
    let mut p = plan(
        &Scope::All,
        &[spec("apt", "bash")],
        &all_installed,
        &["apt".into()],
        &foundation,
    );
    without_protected(&mut p, &|_, _| Some("protected".to_string()));
    assert!(p.batches.is_empty());
    assert!(p.is_empty());
}
