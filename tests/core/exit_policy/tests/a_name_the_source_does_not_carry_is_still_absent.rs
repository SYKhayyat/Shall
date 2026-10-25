use super::*;

/// The other half: the fix must not cost the typo its withdrawal.
#[test]
fn a_name_the_source_does_not_carry_is_still_absent() {
    let p = choco();
    assert!(
        p.names_an_absent_package(&ExitPolicy::haystack(CHOCO_ABSENT_NAME.as_bytes(), b"")),
        "a name the feed answered about is absent, and the line should come back out"
    );
    assert_eq!(
        p.retryability(&ExitPolicy::haystack(CHOCO_ABSENT_NAME.as_bytes(), b"")),
        Retryability::Permanent
    );
}
