use super::*;

/// The sentence itself, because the branch is only half the harm: the advice a user reads
/// must not tell them a moving window will not move.
#[test]
fn a_transient_failure_is_not_advised_as_if_it_were_permanent() {
    let e = boxed(Error::RateLimit(
        "api.github.com is rate limiting this machine and does not reset for 1236s".to_string(),
    ));
    let advice = kept_line_advice(
        why_kept(&e),
        "github:sharkdp/fd",
        std::path::Path::new("modules/imperative.txt"),
    );
    assert!(
        !advice.contains("Nothing classified the failure above"),
        "the advice for a Transient failure is the Unclassified sentence:\n{advice}"
    );
    assert!(
        !advice.contains("the cause is not a passing one"),
        "a rate limit repeats unchanged precisely because it is passing:\n{advice}"
    );
}
