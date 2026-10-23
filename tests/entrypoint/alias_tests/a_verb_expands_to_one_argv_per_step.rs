use super::*;

#[test]
fn a_verb_expands_to_one_argv_per_step() {
    let v = verbs(&[("refresh", &["sync", "upgrade --all"])]);
    let steps = plan_user_verb(&argv(&["shall", "refresh"]), &v, &builtins())
        .unwrap()
        .unwrap();
    assert_eq!(
        steps,
        vec![
            argv(&["shall", "sync"]),
            argv(&["shall", "upgrade", "--all"]),
        ]
    );
}
