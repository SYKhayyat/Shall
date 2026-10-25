use super::*;

#[tokio::test]
async fn a_failing_command_in_a_rhai_hook_fails_the_hook() {
    // `sh` throws where `sh_ok` answers. A hook that swallowed a failed command would report
    // an install as configured when it was not.
    let (_, bad) = commands();
    let hooks = hooks_with(&format!("#rhai\nsh(\"{}\");", bad));
    let err = hooks.run_hook("after_install", "pkg").await.unwrap_err();
    assert!(err.to_string().contains("sh:"), "{}", err);
}
