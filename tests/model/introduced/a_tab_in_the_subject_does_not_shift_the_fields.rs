use super::*;

/// A subject containing a tab must not shift the fields — the split is bounded, so
/// everything after the second tab is the subject, tabs and all.
#[test]
fn a_tab_in_the_subject_does_not_shift_the_fields() {
    let line = "a1111111111111111111111111111111111111111\t2026-03-14\tadd\tripgrep\n";
    let found = introduced_in(line).unwrap();
    assert_eq!(found.date, "2026-03-14");
    assert_eq!(found.subject, "add\tripgrep");
}
