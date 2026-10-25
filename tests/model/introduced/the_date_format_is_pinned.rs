use super::*;

/// The date format is fixed, not the reader's locale: this string is parsed by the test
/// above and read by a person, and `--date=short` is the one that is both.
#[test]
fn the_date_format_is_pinned() {
    assert!(argv("x", &[]).iter().any(|a| a == "--date=short"));
}
