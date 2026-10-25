use super::*;

#[test]
fn a_body_line_without_an_equals_is_an_error() {
    assert!(parse("github {\n  formats deb\n}\n").is_err());
}
