use super::*;

#[tokio::test]
async fn a_rhai_hook_reaches_the_whole_standard_library_vars_has() {
    // Not just the shell: II.6b's "trusted the same as a hook" is a two-way statement, so
    // every provider `vars.shall` has, a hook has. One test per family, not one per member.
    let hooks = hooks_with(
        r#"#rhai
        if weekday() == "" { throw "no clock" }
        if !has_env("PATH") { throw "no environment" }
        if path_exists("/definitely/not/here") { throw "no filesystem" }
        if parse_json(`{"a": 1}`).a != 1 { throw "no json" }
        "#,
    );
    hooks.run_hook("after_install", "pkg").await.unwrap();
}
