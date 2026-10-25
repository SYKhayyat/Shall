use super::*;

#[test]
fn essential_survives_a_missing_priority_field() {
    // `Priority` is optional in dpkg, so an unset field collapses the line to two
    // tokens. Counting from the front would read "base-files" as the priority and drop
    // the package — silently un-protecting something marked Essential: yes.
    let res = parse_essential("yes  base-files\n required login\nno optional jq\n");
    assert!(res.contains(&"base-files".to_string()), "{:?}", res);
    assert!(res.contains(&"login".to_string()), "{:?}", res);
    assert!(!res.contains(&"jq".to_string()));
}
