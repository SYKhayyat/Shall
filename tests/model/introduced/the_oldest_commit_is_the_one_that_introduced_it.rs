use super::*;

/// git lists newest first, so the INTRODUCING commit is the last line. Taking the first
/// would name whoever last edited the line — a different fact, and one that looks correct
/// whenever a line has only ever been touched once.
#[test]
fn the_oldest_commit_is_the_one_that_introduced_it() {
    let found = introduced_in(LOG).expect("a commit");
    assert_eq!(found.date, "2026-03-14");
    assert_eq!(found.subject, "add ripgrep");
    assert_eq!(found.short(), "a111111");
}
