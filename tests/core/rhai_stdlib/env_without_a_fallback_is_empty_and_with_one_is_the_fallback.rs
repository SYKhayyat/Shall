use super::*;

/// **The two arities of `env` differ only for a name that is not set.** `W7` makes this the
/// escape hatch for `SHALL_ROLE` when the hostname cannot say which machine this is, so what
/// a missing variable yields decides which profile a machine gets.
#[test]
fn env_without_a_fallback_is_empty_and_with_one_is_the_fallback() {
    let engine = engine("test");
    let missing = "SHALL_TEST_DEFINITELY_NOT_SET_9E1F";

    let bare: String = engine
        .eval(&format!(r#"env("{missing}")"#))
        .expect("env/1 failed");
    assert_eq!(bare, "", "a name that is not set did not read as empty");

    let fallback: String = engine
        .eval(&format!(r#"env("{missing}", "work")"#))
        .expect("env/2 failed");
    assert_eq!(
        fallback, "work",
        "the fallback was not used for a name that is not set"
    );

    let present: bool = engine
        .eval(&format!(r#"has_env("{missing}")"#))
        .expect("has_env failed");
    assert!(!present, "has_env claimed an unset name is set");
}
