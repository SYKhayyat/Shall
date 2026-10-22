use super::*;

#[test]
fn a_setting_reaches_the_argv_it_is_named_in() {
    let mut cfg = row("conda");
    cfg.install_args = vec!["install".into(), "-n".into(), "{setting.env|base}".into()];
    cfg.resolve_settings(Some(&settings(&[("env", "ml")])))
        .expect("resolves");
    assert_eq!(cfg.install_args, vec!["install", "-n", "ml"]);
}
