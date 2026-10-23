use super::*;

#[test]
fn a_subcommands_own_flag_still_works_on_a_step() {
    // The spec's own example (`refresh = ["sync", "upgrade --all"]`) configures one
    // command, not the run's posture; refusing it would be the fix eating the feature.
    let v = verbs(&[("refresh", &["sync", "upgrade --all"])]);
    let steps = plan_user_verb(&argv(&["shall", "refresh"]), &v, &builtins())
        .unwrap()
        .unwrap();
    assert_eq!(steps[1], argv(&["shall", "upgrade", "--all"]));
}
