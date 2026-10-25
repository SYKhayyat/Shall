use super::*;

/// **A running manager with no lock file on disk holds nothing.** Asking the process list
/// before the filesystem is what made every row read as held on a machine with no `/proc`,
/// because `any_named` answers *yes* there on purpose — the safe direction for clearing, the
/// catastrophic one for waiting.
#[test]
fn a_manager_that_is_running_but_holds_no_lock_file_holds_nothing() {
    assert_eq!(
        held_for(
            "pacman",
            &Fake {
                alive: vec![],
                running: vec!["pacman"]
            },
            &reader(&[]),
        ),
        Held::Free,
    );
}
