use super::*;

/// The other half, and the half that makes the check worth having: a successful command
/// must not be read as a failure.
#[test]
fn a_successful_nimble_listing_is_not_a_failure() {
    assert!(!nimble().signals_failure(&ExitPolicy::haystack(NIMBLE_LIST.as_bytes(), b"")));
}
