use super::*;

#[test]
fn a_bare_version_says_what_was_meant() {
    let err = parse_short(&o(), "2.0").unwrap_err();
    assert_eq!(err.hint.as_deref(), Some("did you mean `@version=2.0`?"));
}
