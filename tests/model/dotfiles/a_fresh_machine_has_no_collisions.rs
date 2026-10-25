use super::*;

#[test]
fn a_fresh_machine_has_no_collisions() {
    let tree = tree_with(&["a.conf"]);
    let p = plan(
        tree.path(),
        Path::new("/home/me"),
        &nothing_exists,
        &nothing_owned,
    )
    .unwrap();
    assert!(p.collisions.is_empty());
}
