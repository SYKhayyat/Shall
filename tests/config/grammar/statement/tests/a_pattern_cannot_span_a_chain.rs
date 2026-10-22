use super::*;

#[test]
fn a_pattern_cannot_span_a_chain() {
    // A pattern is matched against one catalogue and frozen in one regex lock; a chain
    // gives it neither.
    let err = p("apt,cargo:re:^fonts-").unwrap_err();
    assert!(err.what.contains("exactly one backend"), "{}", err.what);
    assert!(p("apt:re:^fonts-").is_ok());
}
