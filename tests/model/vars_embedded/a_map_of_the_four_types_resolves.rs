use super::*;

#[test]
fn a_map_of_the_four_types_resolves() {
    let vars =
        run(r#"#{ role: "travel", cores: 8, ratio: 1.5, gpu: true, tags: ["a", "b"] }"#).unwrap();
    assert_eq!(vars["role"], Value::Str("travel".into()));
    assert_eq!(vars["cores"], Value::Num(8.0));
    assert_eq!(vars["ratio"], Value::Num(1.5));
    assert_eq!(vars["gpu"], Value::Bool(true));
    assert_eq!(
        vars["tags"],
        Value::List(vec![Value::Str("a".into()), Value::Str("b".into())])
    );
}
