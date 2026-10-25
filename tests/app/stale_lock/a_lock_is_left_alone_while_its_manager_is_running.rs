use super::*;

/// **And the case that must never be got wrong.** A pacman is mid-transaction; the lock is
/// its lock. Clearing it here is how a package database is corrupted, and it is the reason
/// the check is a proof rather than a file-exists test.
#[test]
fn a_lock_is_left_alone_while_its_manager_is_running() {
    let found = find(
        &Fake {
            alive: vec![],
            running: vec!["pacman"],
        },
        &reader(&[("/var/lib/pacman/db.lck", "")]),
    );
    assert!(found.stale.is_empty(), "{found:?}");
}
