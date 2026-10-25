use super::*;

/// A lock whose contents ARE the evidence keeps the old answer: unreadable proves nothing
/// about a pid file, and "proves nothing" must not become "go and delete it".
#[test]
fn an_unreadable_pid_lock_is_still_left_alone() {
    let found = find(
        &Fake {
            alive: vec![],
            running: vec![],
        },
        &unreadable(&["/var/cache/dnf/metadata_lock.pid"]),
    );
    assert!(found.stale.is_empty(), "{found:?}");
    assert_eq!(found.left.len(), 1, "{found:?}");
    assert!(found.left[0].because.contains("could not be read"));
}
