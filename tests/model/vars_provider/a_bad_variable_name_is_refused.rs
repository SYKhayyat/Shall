use super::*;

#[test]
fn a_bad_variable_name_is_refused() {
    let origin = Origin::new("vars.js", 0);
    let err = parse_output(r#"{"1bad": "x"}"#, &origin).unwrap_err();
    assert!(err.what.contains("valid variable name"), "{}", err);
}
