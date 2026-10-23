use super::*;

#[test]
fn a_verb_inherits_leading_global_flags_on_every_step() {
    let v = verbs(&[("refresh", &["sync", "check"])]);
    let steps = plan_user_verb(
        &argv(&["shall", "-c", "/c.toml", "refresh"]),
        &v,
        &builtins(),
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        steps,
        vec![
            argv(&["shall", "-c", "/c.toml", "sync"]),
            argv(&["shall", "-c", "/c.toml", "check"]),
        ]
    );
}
