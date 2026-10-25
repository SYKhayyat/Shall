use super::*;

#[test]
fn a_different_value_is_not_already_set() {
    assert!(!already_set("'prefer-light'", "prefer-dark"));
    assert!(!already_set("false", "true"));
}
