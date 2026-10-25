use super::*;

#[test]
fn an_unknown_format_in_a_body_is_an_error_naming_the_real_ones() {
    let err = parse("github {\n  formats = nonsense\n}\n").unwrap_err();
    assert!(err.to_string().contains("nonsense"), "{}", err);
    assert!(err.to_string().contains("tarball"), "{}", err);
}
