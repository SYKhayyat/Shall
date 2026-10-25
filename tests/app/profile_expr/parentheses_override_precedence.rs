use super::*;

#[test]
fn parentheses_override_precedence() {
    // (a | b) & c
    let r = eval_str(
        "(a | b) & c",
        &[("a", &["x"]), ("b", &["y", "z"]), ("c", &["z", "x"])],
    )
    .unwrap();
    assert_eq!(r, vec!["x", "z"]);
}
