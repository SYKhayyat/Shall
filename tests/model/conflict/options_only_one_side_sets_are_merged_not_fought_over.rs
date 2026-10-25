use super::*;

#[test]
fn options_only_one_side_sets_are_merged_not_fought_over() {
    let a = decl("modules/a.txt", 1, &[("version", "1.6")]);
    let b = decl("modules/b.txt", 3, &[("hold", "true")]);
    let out = reconcile("apt:jq", a, b, now()).unwrap();
    assert_eq!(out.options.one("version"), Some("1.6"));
    assert_eq!(out.options.one("hold"), Some("true"));
}
