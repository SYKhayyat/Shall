use super::*;

/// An AUR helper is stopped by pacman's lock, because pacman is what it runs to write.
/// Reading the backend name literally would leave `yay` with no lock and no wait.
#[test]
fn the_aur_helpers_are_answered_by_pacmans_lock() {
    for backend in ["yay", "paru"] {
        assert_eq!(
            held_for(
                backend,
                &Fake {
                    alive: vec![],
                    running: vec!["pacman"]
                },
                &reader(&[("/var/lib/pacman/db.lck", "")]),
            ),
            Held::Live("a `pacman`".into()),
            "{backend} was not answered by pacman's lock"
        );
    }
}
