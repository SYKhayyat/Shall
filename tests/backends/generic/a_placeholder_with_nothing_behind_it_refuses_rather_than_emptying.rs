use super::*;

/// **A key with no value and no default is a refusal.** `conda list -n --json` would hand
/// conda a flag where an environment belongs, and conda answers *something* — which is the
/// silent-wrong-answer shape, not a crash.
#[test]
fn a_placeholder_with_nothing_behind_it_refuses_rather_than_emptying() {
    let mut cfg = row("conda");
    cfg.list_args = vec!["list".into(), "-n".into(), "{setting.env}".into()];
    let err = cfg
        .resolve_settings(None)
        .expect_err("an unresolvable placeholder must not ship");
    let msg = err.to_string();
    assert!(msg.contains("backend_settings.conda"), "{msg}");
    assert!(msg.contains("env"), "{msg}");
}
