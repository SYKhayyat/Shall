use super::*;

#[test]
fn referring_to_a_name_that_does_not_exist_is_an_error() {
    let err = resolve(&[top("tier", "${nosuch}-heavy", 1)]).unwrap_err();
    assert!(err.what.contains("nosuch"), "{}", err);
}
