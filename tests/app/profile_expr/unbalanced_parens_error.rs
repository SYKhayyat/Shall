use super::*;

#[test]
fn unbalanced_parens_error() {
    assert!(eval_str("(a | b", &[]).is_err());
}
