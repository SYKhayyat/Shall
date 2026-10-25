use super::*;

#[test]
fn a_line_that_parses_as_nothing_is_reported_rather_than_read_as_an_empty_machine() {
    // No dash-then-digit anywhere, so nothing is misread as a package — and this used to
    // end there, returning the empty vector that means *this machine has no packages*.
    // It does not mean that. A `pkg info` that printed one line and yielded no package is
    // a listing this parser did not understand, and the planner must not answer it by
    // scheduling every declared package as a fresh install.
    let err = parse_pkg("Updating FreeBSD repository catalogue...")
        .expect_err("one unreadable line is not an empty machine");
    assert_eq!(err.backend, "pkg");
    assert_eq!(err.data_lines, 1);
    assert!(err.sample.starts_with("Updating"), "{err:?}");
}
