use super::*;

#[test]
fn a_malformed_step_is_refused_by_name_instead_of_a_usage_exit() {
    let v = verbs(&[("broken", &["sync --no-such-flag"])]);
    let err = plan_user_verb(&argv(&["shall", "broken"]), &v, &builtins())
        .unwrap()
        .unwrap_err();
    assert!(err.contains("not a valid command line"), "{}", err);
    assert!(err.contains("--no-such-flag"), "{}", err);
}
