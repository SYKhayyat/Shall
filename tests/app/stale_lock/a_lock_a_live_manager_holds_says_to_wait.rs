use super::*;

/// **The case the wait exists for.** Another pacman is mid-transaction. The lock is real,
/// the holder is real, and the only thing that helps is to wait for it — which is not what
/// four retries in three and a half seconds do.
#[test]
fn a_lock_a_live_manager_holds_says_to_wait() {
    let held = held_for(
        "pacman",
        &Fake {
            alive: vec![],
            running: vec!["pacman"],
        },
        &reader(&[("/var/lib/pacman/db.lck", "")]),
    );
    assert_eq!(held, Held::Live("a `pacman`".into()));
}
