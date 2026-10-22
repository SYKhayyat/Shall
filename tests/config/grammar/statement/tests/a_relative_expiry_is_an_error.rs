use super::*;

#[test]
fn a_relative_expiry_is_an_error() {
    // V.38: "2 hours" cannot work in a file — this is why `@lease=2h` was inert.
    let err = p("apt:jq@expires=2h").unwrap_err();
    assert!(err.what.contains("not an absolute date"), "{}", err.what);
    assert!(err.hint.unwrap().contains("2026-07-17T14:00"));
}
