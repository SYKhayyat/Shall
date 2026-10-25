use super::*;

#[test]
fn a_bare_true_or_false_is_a_boolean_not_a_string() {
    let v = resolve(&[top("gpu", "true", 1), top("headless", "false", 2)]).unwrap();
    assert_eq!(v["gpu"], Value::Bool(true));
    assert_eq!(v["headless"], Value::Bool(false));
}
