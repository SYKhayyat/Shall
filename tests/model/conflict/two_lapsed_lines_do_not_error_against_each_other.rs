use super::*;

#[test]
fn two_lapsed_lines_do_not_error_against_each_other() {
    let a = decl("modules/a.txt", 1, &[("expires", "2026-01-01")]);
    let b = decl("modules/b.txt", 2, &[("expires", "2026-02-01")]);
    assert!(reconcile("apt:jq", a, b, now()).is_ok());
}
