use super::*;

#[test]
fn parse_number_rejects_infinity_and_multiple_dots() {
    assert_eq!(parse_number("inf"), None);
    assert_eq!(parse_number("1.2.3"), None);
    assert_eq!(parse_number("1e5"), None);
    assert_eq!(parse_number("-4"), Some(-4.0));
}
