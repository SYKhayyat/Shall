use super::*;

#[test]
fn a_setting_without_a_slash_is_not_schema_key() {
    let err = p("setting:color-scheme@value=prefer-dark").unwrap_err();
    assert!(err.what.contains("SCHEMA/KEY"), "{}", err);
}
