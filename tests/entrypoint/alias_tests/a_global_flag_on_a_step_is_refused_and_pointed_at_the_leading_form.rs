use super::*;

#[test]
fn a_global_flag_on_a_step_is_refused_and_pointed_at_the_leading_form() {
    let v = verbs(&[("refresh", &["check", "sync --dry-run"])]);
    let err = plan_user_verb(&argv(&["shall", "refresh"]), &v, &builtins())
        .unwrap()
        .unwrap_err();
    assert!(err.contains("--dry-run"), "{}", err);
    assert!(err.contains("global flag"), "{}", err);
    assert!(err.contains(&format!("`shall {}`", "refresh")), "{}", err);

    // With leading flags, the suggestion carries them so the fix is copy-pasteable.
    let err = plan_user_verb(
        &argv(&["shall", "-c", "/c.toml", "refresh"]),
        &v,
        &builtins(),
    )
    .unwrap()
    .unwrap_err();
    assert!(err.contains("`shall -c /c.toml refresh`"), "{}", err);
}
