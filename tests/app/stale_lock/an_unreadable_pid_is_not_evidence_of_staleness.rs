use super::*;

/// A pid file with nothing readable in it proves nothing, and "proves nothing" is not
/// "is stale". Half-written by a process that died between `create` and `write` is exactly
/// the moment to be careful rather than clever.
#[test]
fn an_unreadable_pid_is_not_evidence_of_staleness() {
    let found = find(
        &Fake {
            alive: vec![],
            running: vec![],
        },
        &reader(&[("/run/zypp.pid", "not a pid")]),
    );
    assert!(found.stale.is_empty(), "{found:?}");
}
