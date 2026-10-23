use super::*;

#[test]
fn every_posture_spelling_is_refused_wherever_it_hides_on_a_step() {
    for step in [
        "sync --config=/x",
        "sync -c /x",
        "sync -vn",
        "check -y",
        "sync --keep-going",
    ] {
        let v = verbs(&[("r", &[step])]);
        let err = plan_user_verb(&argv(&["shall", "r"]), &v, &builtins())
            .unwrap()
            .unwrap_err();
        assert!(err.contains("global flag"), "{step}: {err}");
    }
}
