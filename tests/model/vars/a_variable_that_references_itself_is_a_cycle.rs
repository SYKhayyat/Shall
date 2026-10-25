use super::*;

#[test]
fn a_variable_that_references_itself_is_a_cycle() {
    let err = resolve(&[top("a", "${a}", 1)]).unwrap_err();
    assert!(err.what.contains("defined in terms of itself"), "{}", err);
}
