use super::*;

#[test]
fn a_variable_defined_only_inside_a_block_is_an_error() {
    // IX.3: otherwise `role` is undefined on every machine that is not the laptop, and
    // `when $role == travel` there has no answer.
    let err = resolve(&[when("role", "travel", 5)]).unwrap_err();
    assert!(
        err.what.contains("only defined inside a `when` block"),
        "{}",
        err
    );
    assert!(err.to_string().contains("vars:5"), "{}", err);
}
