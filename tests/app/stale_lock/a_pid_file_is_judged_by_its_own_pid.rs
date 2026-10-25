use super::*;

/// A pid file answers for itself, and the answer is about that pid — not about whether any
/// process of that name exists. Two dnf runs on one machine is not a thing, but a *reused*
/// pid belonging to something else is, and `pid_alive` is what the lock's own claim means.
#[test]
fn a_pid_file_is_judged_by_its_own_pid() {
    let held = find(
        &Fake {
            alive: vec![4242],
            running: vec![],
        },
        &reader(&[("/var/cache/dnf/metadata_lock.pid", "4242\n")]),
    );
    assert!(held.stale.is_empty(), "a live pid holds its lock: {held:?}");

    let stale = find(
        &Fake {
            alive: vec![1],
            running: vec!["dnf"],
        },
        &reader(&[("/var/cache/dnf/metadata_lock.pid", "4242\n")]),
    );
    assert_eq!(stale.stale.len(), 1, "{stale:?}");
    assert!(
        stale.stale[0].because.contains("4242"),
        "the reason has to name the pid it looked for: {}",
        stale.stale[0].because
    );
}
