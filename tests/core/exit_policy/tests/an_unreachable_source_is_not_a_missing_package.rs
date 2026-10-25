use super::*;

/// A manager that could not reach its index has not looked the name up.
///
/// Both fixtures are real `choco install` runs measured on 2026-08-02, and they end with
/// the same sentence — `The package was not found with the source(s) listed.` One of them
/// was pointed at a port nothing is listening on. Only the connection lines above it say
/// which is which, so believing the sentence on its own deletes a declaration whose
/// package exists, on nothing worse than a dropped VPN.
#[test]
fn an_unreachable_source_is_not_a_missing_package() {
    let p = choco();
    assert!(
        !p.names_an_absent_package(&ExitPolicy::haystack(
            CHOCO_SOURCE_UNREACHABLE.as_bytes(),
            b""
        )),
        "an unreachable feed read as `no such package`, which withdraws the line from the \
         user's config files"
    );
    assert_eq!(
        p.retryability(&ExitPolicy::haystack(
            CHOCO_SOURCE_UNREACHABLE.as_bytes(),
            b""
        )),
        Retryability::Transient,
        "an unreachable feed is worth another attempt"
    );
    // The failure itself is still a failure — this must not become a silent success.
    assert!(p.signals_failure(&ExitPolicy::haystack(
        CHOCO_SOURCE_UNREACHABLE.as_bytes(),
        b""
    )));
}
