use super::*;

#[test]
fn a_policy_with_no_markers_classifies_nothing() {
    let p = ExitPolicy::default();
    assert_eq!(
        p.retryability(&ExitPolicy::haystack(b"anything", b"")),
        Retryability::Unknown
    );
    assert!(!p.signals_failure(&ExitPolicy::haystack(
        b"Couldn't find manifest for 'x'.",
        b""
    )));
    assert!(!p.is_benign(Some(3010)));
}
