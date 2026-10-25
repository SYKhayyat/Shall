use super::*;

#[test]
fn union_infix_dedups_in_order() {
    let r = eval_str("a | b", &[("a", &["x", "y"]), ("b", &["y", "z"])]).unwrap();
    assert_eq!(r, vec!["x", "y", "z"]);
}
