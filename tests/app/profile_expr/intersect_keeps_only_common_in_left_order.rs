use super::*;

#[test]
fn intersect_keeps_only_common_in_left_order() {
    let r = eval_str("a & b", &[("a", &["x", "y", "z"]), ("b", &["z", "x"])]).unwrap();
    assert_eq!(r, vec!["x", "z"]);
}
