use super::*;

#[test]
fn value_equality_has_no_cross_type_coercion() {
    assert!(Value::Num(1.0).equals(&Value::Num(1.0)));
    assert!(!Value::Str("1".into()).equals(&Value::Num(1.0)));
    assert!(!Value::Bool(true).equals(&Value::Str("true".into())));
    assert!(Value::Str("Linux".into()).equals(&Value::Str("linux".into())));
}
