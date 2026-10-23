use super::*;

#[test]
fn a_row_of_the_same_shape_that_is_not_a_dependency_is_left_alone() {
    // `Optional Deps` and `Conflicts With` sit in the same column and are not dependencies;
    // matching by prefix on `Depends` would have taken `Optional Deps` too once the width
    // was right.
    let deps = parse_depends_on(SI_JQ);
    assert!(!deps.iter().any(|d| d == "None"), "{:?}", deps);
    assert!(!deps.iter().any(|d| d == "Command-line"), "{:?}", deps);
}
