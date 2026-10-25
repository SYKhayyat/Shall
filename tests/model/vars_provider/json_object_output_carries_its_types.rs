use super::*;

#[test]
fn json_object_output_carries_its_types() {
    let origin = Origin::new("vars.js", 0);
    let (vars, origins) = parse_output(
        r#"{"role": "travel", "cores": 8, "gpu": true, "tags": ["a", "b"]}"#,
        &origin,
    )
    .unwrap();
    assert_eq!(vars["role"], Value::Str("travel".into()));
    assert_eq!(vars["cores"], Value::Num(8.0));
    assert_eq!(vars["gpu"], Value::Bool(true));
    assert_eq!(
        vars["tags"],
        Value::List(vec![Value::Str("a".into()), Value::Str("b".into())])
    );
    // JSON has no lines, so a variable's origin is the provider file itself.
    assert_eq!(origins["role"].to_string(), "vars.js");
}
