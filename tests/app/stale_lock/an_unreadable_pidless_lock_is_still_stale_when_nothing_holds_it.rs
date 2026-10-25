use super::*;

/// The same lock, seen by a user who may not open it — which is every Shall run that is not
/// root, and pacman's `db.lck` is mode 0000 and owned by root.
///
/// Measured before this held: the arch harness (the one image that runs unprivileged)
/// SIGKILLed a sync, and the three checks that follow all failed on
/// `could not lock database: File exists`. `heal` had said *"left it alone — it is there
/// and could not be read"* about a file with nothing in it to read.
#[test]
fn an_unreadable_pidless_lock_is_still_stale_when_nothing_holds_it() {
    let found = find(
        &Fake {
            alive: vec![],
            running: vec![],
        },
        &unreadable(&["/var/lib/pacman/db.lck"]),
    );
    assert_eq!(found.stale.len(), 1, "{found:?}");
    assert_eq!(found.stale[0].holder, "pacman");
    assert!(found.left.is_empty(), "{found:?}");
}
