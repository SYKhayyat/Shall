use super::*;

/// GRADER round 5, 2026-07-30 — RED.
///
/// `error.rs` classifies a rate limit `Transient`, and says why in as many words:
/// *"The whole point of a rate limit is that the window moves."* `why_kept` branches on
/// `Refused`, `Exhausted` and name-absence and then falls through to `Unclassified`, so the
/// user is told **"Nothing classified the failure above"** about a failure this program
/// classified three lines away — and then told *"if it repeats unchanged the cause is not a
/// passing one"*, which is exactly backwards: a rate limit repeats unchanged *because* it is
/// passing.
///
/// Observed live on the macOS runner, with the window in the line above the advice:
///
///     Error: API rate limit: api.github.com is rate limiting this machine and does not
///     reset for 1236s, past the 30s ceiling. …
///      WARN `github:sharkdp/fd` is still declared in …, so `sync` will try it again.
///           Nothing classified the failure above, …
///
/// It costs two red CI jobs: the sweep harness tests transience by retrying immediately,
/// which cannot succeed inside a 1236-second window, so it scores `defect`, the macOS leg
/// goes red, and the real-lifecycle ratchet falls 8 -> 7 and goes red behind it.
#[test]
fn a_transient_failure_is_not_reported_as_unclassified() {
    let e = boxed(Error::RateLimit(
        "api.github.com is rate limiting this machine and does not reset for 1236s".to_string(),
    ));
    assert_eq!(
        e.downcast_ref::<Error>().map(|x| x.retryability()),
        Some(Retryability::Transient),
        "this fixture is not transient, so it does not test the distinction"
    );

    let why = why_kept(&e);
    assert_ne!(
        why,
        WhyKept::Unclassified,
        "a failure `Error::retryability()` calls Transient is routed to the one branch whose \
         text says nothing classified it"
    );
}
