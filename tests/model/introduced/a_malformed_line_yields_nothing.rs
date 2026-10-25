use super::*;

/// Garbage is not half-parsed into a plausible-looking commit.
#[test]
fn a_malformed_line_yields_nothing() {
    assert_eq!(introduced_in("not a log line\n"), None);
    assert_eq!(introduced_in("\t2026-03-14\tsubject\n"), None);
}
