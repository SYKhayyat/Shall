use super::*;

#[tokio::test]
async fn a_rhai_hook_can_reach_the_shell() {
    // The bug: this arm had `print` and nothing else, so the shipped example config's
    // `exec("systemctl enable docker")` — and any `sh` — was a call to a function no engine
    // in this binary had ever registered.
    let (ok, _) = commands();
    let hooks = hooks_with(&format!("#rhai\nlet ran = sh_ok(\"{}\");", ok));
    hooks.run_hook("after_install", "pkg").await.unwrap();
}
