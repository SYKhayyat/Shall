use super::*;

/// The defect this mechanism exists for: scoop's single phrasing marker caught a missing
/// manifest and nothing else, so removing something that was never installed reported
/// success.
#[test]
fn scoop_sees_a_failure_that_is_not_a_missing_manifest() {
    let p = scoop();
    assert!(
        p.signals_failure(&ExitPolicy::haystack(
            SCOOP_UNINSTALL_MISSING.as_bytes(),
            b""
        )),
        "scoop uninstall of a package that is not installed read as success"
    );
    assert!(p.signals_failure(&ExitPolicy::haystack(
        b"Couldn't find manifest for 'nope'.",
        b""
    )));
    assert!(!p.signals_failure(&ExitPolicy::haystack(
        b"'jq' (1.7.1) was installed successfully!",
        b""
    )));
}
