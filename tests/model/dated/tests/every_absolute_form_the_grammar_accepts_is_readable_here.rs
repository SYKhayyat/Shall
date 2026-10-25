use super::*;

#[test]
fn every_absolute_form_the_grammar_accepts_is_readable_here() {
    // The grammar's validator and this reader must accept the same set, or a line the
    // grammar allows silently becomes undated and counts forever.
    assert!(parse_absolute("2026-07-17T14:00:00Z").is_some());
    assert!(parse_absolute("2026-07-17T14:00:00").is_some());
    assert!(parse_absolute("2026-07-17T14:00").is_some());
    assert!(parse_absolute("2026-07-17").is_some());
    assert!(parse_absolute("2h").is_none());
}
