use super::*;

#[test]
fn a_verb_takes_no_arguments() {
    let v = verbs(&[("refresh", &["sync"])]);
    let err = plan_user_verb(&argv(&["shall", "refresh", "--dry-run"]), &v, &builtins())
        .unwrap()
        .unwrap_err();
    assert!(err.contains("takes no arguments"), "{}", err);
}
