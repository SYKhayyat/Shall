use super::*;

#[test]
fn scoop_reports_a_missing_manifest_as_a_failure_and_never_retries_it() {
    let p = scoop();
    assert!(p.signals_failure(&ExitPolicy::haystack(
        b"Couldn't find manifest for 'nope'.",
        b""
    )));
    assert!(p.signals_failure(&ExitPolicy::haystack(
        b"",
        b"couldn't find manifest for 'nope'."
    )));
    assert!(!p.signals_failure(&ExitPolicy::haystack(b"Installing 'jq'...", b"")));
    assert_eq!(
        p.retryability(&ExitPolicy::haystack(
            b"Couldn't find manifest for 'nope'.",
            b""
        )),
        Retryability::Permanent
    );
}
