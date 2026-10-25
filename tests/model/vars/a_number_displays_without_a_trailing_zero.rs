use super::*;

#[test]
fn a_number_displays_without_a_trailing_zero() {
    assert_eq!(Value::Num(3.0).to_string(), "3");
    assert_eq!(Value::Num(1.5).to_string(), "1.5");
}
