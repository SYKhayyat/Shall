use super::*;

#[test]
fn a_pin_that_answers_to_both_spellings_is_an_error_naming_both() {
    let err = one_release(
        "sharkdp/fd",
        "10.2.0",
        Some(release("10.2.0")),
        Some(release("v10.2.0")),
    )
    .unwrap_err()
    .to_string();
    assert!(err.contains("10.2.0"), "{}", err);
    assert!(err.contains("v10.2.0"), "{}", err);
}
