use super::*;

/// The rule for reading the manager's half of the name, in one place.
#[test]
fn the_unqualified_half_is_everything_after_the_last_slash() {
    assert_eq!(unqualified("app-misc/jq"), "jq");
    assert_eq!(unqualified("jq"), "jq");
    assert_eq!(unqualified("a/b/c"), "c");
    assert_eq!(unqualified(""), "");
}
