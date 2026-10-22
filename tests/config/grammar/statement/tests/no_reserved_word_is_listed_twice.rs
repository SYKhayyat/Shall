use super::*;

/// Reserved words are unique. Two entries for one word means the second is unreachable:
/// both the dispatch loop and `bare_keyword` take the first match.
#[test]
fn no_reserved_word_is_listed_twice() {
    let mut seen: Vec<&str> = reserved_words().iter().map(|(w, _)| *w).collect();
    let before = seen.len();
    seen.sort_unstable();
    seen.dedup();
    assert_eq!(before, seen.len(), "a reserved word is listed twice");
}
