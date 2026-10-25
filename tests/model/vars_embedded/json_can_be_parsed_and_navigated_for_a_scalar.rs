use super::*;

#[test]
fn json_can_be_parsed_and_navigated_for_a_scalar() {
    let vars = run(r#"#{ v: parse_json(`{"a": {"b": 42}}`).a.b }"#).unwrap();
    assert_eq!(vars["v"], Value::Num(42.0));
}
