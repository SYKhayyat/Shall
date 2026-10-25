use super::*;

/// A lock that is not there is not a problem. The common case, and it must cost nothing.
#[test]
fn a_machine_with_no_locks_has_nothing_to_clear() {
    let found = find(
        &Fake {
            alive: vec![],
            running: vec![],
        },
        &reader(&[]),
    );
    assert!(found.stale.is_empty());
    // And nothing to say about it either: the ordinary machine gets no output at all.
    assert!(found.left.is_empty());
}
