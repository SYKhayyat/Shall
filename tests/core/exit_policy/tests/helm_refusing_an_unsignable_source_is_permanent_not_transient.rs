use super::*;

#[test]
fn helm_refusing_an_unsignable_source_is_permanent_not_transient() {
    assert_eq!(
        helm().retryability(&ExitPolicy::haystack(b"", HELM_UNVERIFIABLE.as_bytes())),
        Retryability::Permanent
    );
}
