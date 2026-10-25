use super::*;

#[test]
fn one_name_cannot_have_two_defaults() {
    let err = resolve(&[top("role", "a", 1), top("role", "b", 2)]).unwrap_err();
    assert!(err.what.contains("defined twice"), "{}", err);
}
