use super::*;

/// A backend with no lock in the table is not made to wait for one.
#[test]
fn a_backend_with_no_manager_lock_is_always_free() {
    assert_eq!(
        held_for(
            "npm",
            &Fake {
                alive: vec![],
                running: vec!["pacman"]
            },
            &reader(&[("/var/lib/pacman/db.lck", "")]),
        ),
        Held::Free
    );
    assert!(!says_the_lock_is_taken(
        "npm",
        "unable to lock database: File exists"
    ));
}
