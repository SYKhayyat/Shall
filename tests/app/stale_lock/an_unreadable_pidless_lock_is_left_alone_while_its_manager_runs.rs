use super::*;

/// And the guard that stops it becoming "delete every lock you cannot read": a live holder
/// still wins, unreadable or not.
#[test]
fn an_unreadable_pidless_lock_is_left_alone_while_its_manager_runs() {
    let found = find(
        &Fake {
            alive: vec![],
            running: vec!["pacman"],
        },
        &unreadable(&["/var/lib/pacman/db.lck"]),
    );
    assert!(found.stale.is_empty(), "{found:?}");
    assert_eq!(found.left.len(), 1, "{found:?}");
    assert!(found.left[0].because.contains("is running, so it is held"));
}
