use super::*;

#[test]
fn a_name_that_is_neither_builtin_nor_verb_is_left_alone() {
    let v = verbs(&[("refresh", &["sync"])]);
    assert!(plan_user_verb(&argv(&["shall", "whatever"]), &v, &builtins()).is_none());
}
