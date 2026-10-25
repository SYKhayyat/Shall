use super::*;

/// The case this exists for: a killed run, and pacman's empty lock left behind.
#[test]
fn a_lock_with_no_pid_is_stale_when_its_manager_is_not_running() {
    let found = find(
        &Fake {
            alive: vec![],
            running: vec![],
        },
        &reader(&[("/var/lib/pacman/db.lck", "")]),
    );
    assert_eq!(found.stale.len(), 1, "{found:?}");
    assert_eq!(found.stale[0].holder, "pacman");
    assert!(found.stale[0].because.contains("no `pacman` is running"));
}
