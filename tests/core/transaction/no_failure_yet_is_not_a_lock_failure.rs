use super::*;

/// Nothing has failed yet on the first attempt, so there is nothing to classify.
#[test]
fn no_failure_yet_is_not_a_lock_failure() {
    let verdict = lock_wait_verdict(
        &None,
        "pacman",
        Duration::from_secs(300),
        Duration::from_secs(300),
        &|_| panic!("there is no error to have been about a lock"),
    );
    assert!(matches!(verdict, LockWait::Backoff), "{verdict:?}");
}
