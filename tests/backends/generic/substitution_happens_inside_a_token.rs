use super::*;

/// The value substitutes *inside* the token, so one argument comes out — not two, and not a
/// token with a space in it that the shell would never see because there is no shell.
#[test]
fn substitution_happens_inside_a_token() {
    let mut cfg = row("flatpak");
    cfg.install_args = vec!["install".into(), "--{setting.scope|system}".into()];
    cfg.resolve_settings(Some(&settings(&[("scope", "user")])))
        .expect("resolves");
    assert_eq!(cfg.install_args, vec!["install", "--user"]);
}
