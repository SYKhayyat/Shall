use super::*;

#[test]
fn a_json_object_value_is_refused() {
    let origin = Origin::new("vars.js", 0);
    let err = parse_output(r#"{"x": {"nested": 1}}"#, &origin).unwrap_err();
    assert!(err.what.contains("object"), "{}", err);
}
