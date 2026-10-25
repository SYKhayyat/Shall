use super::*;

#[test]
fn an_unclosed_body_is_an_error_naming_the_backend() {
    let err = parse("github {\n  formats = deb\n").unwrap_err();
    assert!(err.to_string().contains("github"), "{}", err);
}
