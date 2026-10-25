use super::*;

/// U25: several trees are fine; two claiming one destination is an error naming both.
#[test]
fn two_trees_claiming_one_destination_are_named_together() {
    let work = tree_with(&[".gitconfig", "work-only"]);
    let home_tree = tree_with(&[".gitconfig", "home-only"]);
    let home = Path::new("/home/me");
    let a = plan(work.path(), home, &nothing_exists, &nothing_owned).unwrap();
    let b = plan(home_tree.path(), home, &nothing_exists, &nothing_owned).unwrap();

    let clashes = conflicting_destinations(&[("./work".into(), a), ("./home".into(), b)]);
    assert_eq!(clashes.len(), 1, "{:?}", clashes);
    let (dest, trees) = clashes.iter().next().unwrap();
    assert!(dest.ends_with(".gitconfig"));
    assert_eq!(trees, &vec!["./home".to_string(), "./work".to_string()]);
}
