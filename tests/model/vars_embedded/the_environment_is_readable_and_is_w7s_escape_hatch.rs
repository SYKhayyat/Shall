use super::*;

#[test]
fn the_environment_is_readable_and_is_w7s_escape_hatch() {
    std::env::set_var("SHALL_TEST_ROLE", "work");
    let vars = run(
        r#"#{ role: env("SHALL_TEST_ROLE"), missing: env("SHALL_NOPE", "default"), present: has_env("SHALL_TEST_ROLE") }"#,
    )
    .unwrap();
    std::env::remove_var("SHALL_TEST_ROLE");
    assert_eq!(vars["role"], Value::Str("work".into()));
    assert_eq!(vars["missing"], Value::Str("default".into()));
    assert_eq!(vars["present"], Value::Bool(true));
}
