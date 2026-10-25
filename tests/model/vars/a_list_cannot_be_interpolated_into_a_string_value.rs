use super::*;

#[test]
fn a_list_cannot_be_interpolated_into_a_string_value() {
    let err = resolve(&[top("tags", "[a, b]", 1), top("label", "x-${tags}", 2)]).unwrap_err();
    assert!(err.what.contains("list"), "{}", err);
}
