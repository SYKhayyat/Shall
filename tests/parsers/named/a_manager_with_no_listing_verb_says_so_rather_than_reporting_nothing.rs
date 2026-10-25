use super::*;

/// `cannot_list` is an error, not an empty machine. This is the assertion that stops it
/// being "simplified" back into `Ok(vec![])`.
#[test]
fn a_manager_with_no_listing_verb_says_so_rather_than_reporting_nothing() {
    let p = NamedParser::new("stack", installed("cannot_list").unwrap(), None, None);
    let err = p
        .parse_installed("anything at all")
        .expect_err("a manager with no listing verb cannot report an empty one");
    assert_eq!(err.backend, "stack");
}
