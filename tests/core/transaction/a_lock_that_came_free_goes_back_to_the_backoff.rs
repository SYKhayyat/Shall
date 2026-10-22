use super::*;

/// The holder let go between the failure and the question. That is an ordinary race, and the
/// ordinary backoff is the right answer â€” not a wait for a lock that is already free.
#[test]
fn a_lock_that_came_free_goes_back_to_the_backoff() {
    let verdict = lock_wait_verdict(
        &lock_failure(PACMAN_SAID),
        "pacman",
        Duration::from_secs(300),
        Duration::from_secs(300),
        &|_| Held::Free,
    );
    assert!(matches!(verdict, LockWait::Backoff), "{verdict:?}");
}
