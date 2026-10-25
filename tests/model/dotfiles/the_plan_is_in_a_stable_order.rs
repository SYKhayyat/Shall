use super::*;

/// The plan is read by a human before anything is written, so it must not reorder itself
/// between runs.
#[test]
fn the_plan_is_in_a_stable_order() {
    let tree = tree_with(&["z", "a", "m/b", "m/a"]);
    let home = Path::new("/home/me");
    let first = plan(tree.path(), home, &nothing_exists, &nothing_owned).unwrap();
    let again = plan(tree.path(), home, &nothing_exists, &nothing_owned).unwrap();
    assert_eq!(first, again);
    let rels: Vec<String> = first
        .placements
        .iter()
        .map(|x| x.relative.to_string_lossy().replace('\\', "/"))
        .collect();
    assert_eq!(rels, vec!["a", "m/a", "m/b", "z"]);
}
