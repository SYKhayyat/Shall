use super::*;

/// And the half that keeps the marker honest: the same summary line on its own — a
/// reachable index that really does not carry the rock — is not retried forever. It
/// answers `Unknown`, which hard-fails and withdraws nothing.
#[test]
fn a_luarocks_summary_without_a_download_failure_is_not_retried() {
    assert_eq!(
        luarocks().retryability(&ExitPolicy::haystack(
            b"",
            b"Error: No results matching query were found for Lua 5.4."
        )),
        Retryability::Unknown
    );
}
