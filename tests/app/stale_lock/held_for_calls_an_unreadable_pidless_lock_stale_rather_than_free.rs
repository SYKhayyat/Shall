use super::*;

/// The waiter's half of the same conflation: `held_for` filtered on "could the file be
/// read", so an unreadable stale lock reported `Free` and the retry loop backed off four
/// times into `exhausted` instead of sending the user to `heal`.
#[test]
fn held_for_calls_an_unreadable_pidless_lock_stale_rather_than_free() {
    let held = held_for(
        "pacman",
        &Fake {
            alive: vec![],
            running: vec![],
        },
        &unreadable(&["/var/lib/pacman/db.lck"]),
    );
    assert!(matches!(held, Held::Stale(_)), "{held:?}");
}
