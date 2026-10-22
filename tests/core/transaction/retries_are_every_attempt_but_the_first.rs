use super::*;

/// A run that never retried reports no retries.
///
/// `attempt` counts tries and `retries` counts the ones after the first, and the three
/// `TaskResult` sites that used to spell that out inline each carried the same untested
/// subtraction. There is one now, and this is what stops it reading `attempt + 1`.
#[test]
fn retries_are_every_attempt_but_the_first() {
    assert_eq!(retries_behind(1), 0, "the first try is not a retry");
    assert_eq!(retries_behind(2), 1);
    assert_eq!(retries_behind(4), 3);
}
