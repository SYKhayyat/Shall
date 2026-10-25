use super::*;

/// The pairing the test above used to hide: an unreachable index outranks the "not found"
/// the manager prints *because* it was unreachable.
#[test]
fn transient_wins_over_absent() {
    let p = apt();
    let both = b"E: Could not get lock; E: Unable to locate package nope" as &[u8];
    assert_eq!(
        p.retryability(&ExitPolicy::haystack(b"", both)),
        Retryability::Transient
    );
    assert!(!p.names_an_absent_package(&ExitPolicy::haystack(b"", both)));
}
