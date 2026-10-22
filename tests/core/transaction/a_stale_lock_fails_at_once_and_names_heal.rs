use super::*;

/// A lock nothing holds is a corpse, and waiting for it never ends. It fails at once, and
/// the message names the command that clears it rather than three more retries.
#[test]
fn a_stale_lock_fails_at_once_and_names_heal() {
    let verdict = lock_wait_verdict(
        &lock_failure(PACMAN_SAID),
        "pacman",
        Duration::from_secs(300),
        Duration::from_secs(300),
        &|_| Held::Stale("/var/lib/pacman/db.lck".into()),
    );
    let LockWait::Hopeless(err) = verdict else {
        panic!("waiting on a lock nothing holds never ends: {verdict:?}");
    };
    let said = err.to_string();
    assert!(said.contains("shall heal"), "{said}");
    assert!(said.contains("db.lck"), "the file has to be named: {said}");
    assert_eq!(err.retryability(), Retryability::Exhausted);
}
