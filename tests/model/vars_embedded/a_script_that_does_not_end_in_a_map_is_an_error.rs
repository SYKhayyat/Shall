use super::*;

#[test]
fn a_script_that_does_not_end_in_a_map_is_an_error() {
    let err = run("42").unwrap_err();
    assert!(err.hint.as_deref().unwrap_or("").contains("map"), "{}", err);
}
