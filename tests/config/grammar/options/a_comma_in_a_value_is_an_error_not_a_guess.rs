use super::*;

#[test]
fn a_comma_in_a_value_is_an_error_not_a_guess() {
    // II.2. `>=1.0,<2.0` could be one value or two and both readings are plausible,
    // so the parser refuses rather than picking one.
    let err = parse_short(&o(), "version=>=1.0,<2.0").unwrap_err();
    let hint = err.hint.unwrap();
    assert!(hint.contains("block form"), "{}", hint);
}
