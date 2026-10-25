use super::*;

#[test]
fn trees_that_do_not_overlap_are_not_a_conflict() {
    let a = tree_with(&["one"]);
    let b = tree_with(&["two"]);
    let home = Path::new("/home/me");
    let pa = plan(a.path(), home, &nothing_exists, &nothing_owned).unwrap();
    let pb = plan(b.path(), home, &nothing_exists, &nothing_owned).unwrap();
    assert!(conflicting_destinations(&[("a".into(), pa), ("b".into(), pb)]).is_empty());
}
