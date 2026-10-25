use super::*;

#[test]
fn trailing_junk_errors() {
    assert!(eval_str("a b", &[]).is_err());
}
