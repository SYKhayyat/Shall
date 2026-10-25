use super::*;

/// A repo with no history for this line has no answer, and must not invent one.
#[test]
fn no_history_is_no_answer() {
    assert_eq!(introduced_in(""), None);
    assert_eq!(introduced_in("\n  \n"), None);
}
