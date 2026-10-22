use super::*;

/// **The machine is not consulted for a failure that is not about a lock.** A wait on every
/// failed install would be a hang on every typo, and the `/proc` scan would be paid on
/// every one of them.
#[test]
fn a_failure_that_is_not_about_a_lock_never_asks_the_machine() {
    let asked = std::cell::Cell::new(false);
    let verdict = lock_wait_verdict(
        &lock_failure("`pacman` failed (exit 1): error: target not found: qqqq"),
        "pacman",
        Duration::from_secs(300),
        Duration::from_secs(300),
        &|_| {
            asked.set(true);
            Held::Live("a `pacman`".into())
        },
    );
    assert!(matches!(verdict, LockWait::Backoff), "{verdict:?}");
    assert!(
        !asked.get(),
        "the machine was scanned over a missing package"
    );
}
