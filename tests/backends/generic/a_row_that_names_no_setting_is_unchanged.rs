use super::*;

/// A row with no placeholders is untouched, which is every other backend in the tree.
#[test]
fn a_row_that_names_no_setting_is_unchanged() {
    let mut cfg = row("apt");
    cfg.install_args = vec!["install".into(), "-y".into()];
    cfg.resolve_settings(None).expect("resolves");
    assert_eq!(cfg.install_args, vec!["install", "-y"]);
}
