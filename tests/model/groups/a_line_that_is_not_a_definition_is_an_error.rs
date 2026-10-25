use super::*;

#[test]
fn a_line_that_is_not_a_definition_is_an_error() {
    assert!(Groups::parse("apt dnf cargo\n").is_err());
    assert!(Groups::parse("= apt\n").is_err());
    assert!(Groups::parse("tools =\n").is_err());
}
