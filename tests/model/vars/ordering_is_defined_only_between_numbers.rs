use super::*;

#[test]
fn ordering_is_defined_only_between_numbers() {
    use std::cmp::Ordering;
    assert_eq!(
        Value::Num(9.0).order(&Value::Num(10.0)),
        Some(Ordering::Less)
    );
    assert_eq!(Value::Str("9".into()).order(&Value::Str("10".into())), None);
    assert_eq!(Value::Num(1.0).order(&Value::Str("1".into())), None);
}
