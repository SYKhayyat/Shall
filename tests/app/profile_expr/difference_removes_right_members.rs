use super::*;

#[test]
fn difference_removes_right_members() {
    let r = eval_str("a \\ b", &[("a", &["x", "y", "z"]), ("b", &["y"])]).unwrap();
    assert_eq!(r, vec!["x", "z"]);
}
