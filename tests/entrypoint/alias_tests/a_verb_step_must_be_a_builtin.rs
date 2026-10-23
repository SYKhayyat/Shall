use super::*;

#[test]
fn a_verb_step_must_be_a_builtin() {
    let v = verbs(&[("evil", &["rm -rf /"])]);
    let err = plan_user_verb(&argv(&["shall", "evil"]), &v, &builtins())
        .unwrap()
        .unwrap_err();
    assert!(err.contains("not a built-in"), "{}", err);
    assert!(err.contains("exec:"), "{}", err);
}
