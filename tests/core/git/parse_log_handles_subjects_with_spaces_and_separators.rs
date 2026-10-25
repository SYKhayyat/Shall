use super::*;

#[test]
fn parse_log_handles_subjects_with_spaces_and_separators() {
    let raw = "abc123\u{1f}abc\u{1f}2026-07-15\u{1f}N\u{1f}\u{1f}feat: add ripgrep, remove nano\n\
               def456\u{1f}def\u{1f}2026-07-14\u{1f}G\u{1f}Shaul <s@example.com>\u{1f}initial commit";
    let commits = parse_log(raw);
    assert_eq!(commits.len(), 2);
    assert_eq!(commits[0].hash, "abc123");
    assert_eq!(commits[0].short, "abc");
    assert_eq!(commits[0].date, "2026-07-15");
    assert_eq!(commits[0].subject, "feat: add ripgrep, remove nano");
    assert_eq!(commits[0].signature, Signature::Unsigned);
    assert_eq!(commits[1].subject, "initial commit");
    assert_eq!(
        commits[1].signature,
        Signature::Good("Shaul <s@example.com>".to_string())
    );
}
