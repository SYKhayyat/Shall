use super::*;

#[test]
fn looks_like_expression_detects_operators() {
    assert!(looks_like_expression("a | b"));
    assert!(looks_like_expression("intersect(a, b)"));
    assert!(looks_like_expression("(a) & c"));
    assert!(!looks_like_expression("apt:ripgrep"));
    assert!(!looks_like_expression("include base"));
    assert!(!looks_like_expression("-vim"));
    // A package name with a '+' must not be mistaken for an expression.
    assert!(!looks_like_expression("apt:g++"));
}
