use super::*;

/// And the case waiting would never end: the lock outlived its holder. Waiting is forever,
/// so the answer has to be the other one.
#[test]
fn a_lock_nothing_holds_says_it_is_stale_and_names_the_file() {
    let held = held_for(
        "pacman",
        &Fake {
            alive: vec![],
            running: vec![],
        },
        &reader(&[("/var/lib/pacman/db.lck", "")]),
    );
    assert_eq!(held, Held::Stale("/var/lib/pacman/db.lck".into()));
}
