use super::*;

/// U23: a destination holding the user's own file is a collision, reported before anything
/// is written. One Shall already owns is not.
#[test]
fn a_destination_the_user_owns_is_a_collision_and_ours_is_not() {
    let tree = tree_with(&["a.conf", "b.conf"]);
    let home = Path::new("/home/me");
    let exists = |_: &Path| true;
    let owned = |p: &Path| p.ends_with("b.conf");

    let plan = plan(tree.path(), home, &exists, &owned).unwrap();
    assert_eq!(plan.collisions.len(), 1);
    assert!(plan.collisions[0].ends_with("a.conf"));
    // Both are still placed — a collision is a thing to confirm, not a file to drop.
    assert_eq!(plan.placements.len(), 2);
}
