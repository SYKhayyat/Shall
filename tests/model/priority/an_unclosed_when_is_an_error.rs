use super::*;

#[test]
fn an_unclosed_when_is_an_error() {
    assert!(parse("when host == laptop {\n  cargo\n").is_err());
}
