use super::*;

#[test]
fn a_package_named_like_a_flag_is_not_confused_for_one() {
    // The name is excluded from the flags before matching, so a package called "yes"
    // is only protected if its own flags say so.
    assert!(parse_essential("no optional yes\n").is_empty());
    assert_eq!(parse_essential("yes required yes\n"), vec!["yes"]);
}
