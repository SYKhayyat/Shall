use super::*;

/// A live holder is a queue to join. Waiting is the only thing that helps, and it is what
/// Shall already does for its own lock.
#[test]
fn a_live_holder_is_waited_for() {
    let verdict = lock_wait_verdict(
        &lock_failure(PACMAN_SAID),
        "pacman",
        Duration::from_secs(300),
        Duration::from_secs(300),
        &|_| Held::Live("a `pacman`".into()),
    );
    assert!(
        matches!(&verdict, LockWait::Wait(who) if who.contains("pacman")),
        "{verdict:?}"
    );
}
