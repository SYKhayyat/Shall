use super::*;

#[test]
fn an_unclosed_use_paren_is_an_error() {
    assert!(p("use m(user=shaul").is_err());
}
