use super::*;

#[test]
fn param_names_must_be_identifiers() {
    assert!(p("param 9lives").is_err());
    assert!(p("param a-b").is_err());
    assert!(p("param").is_err());
}
