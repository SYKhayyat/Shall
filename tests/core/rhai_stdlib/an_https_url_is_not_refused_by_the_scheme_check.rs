use super::*;

/// The refusal is about the scheme and nothing else — an https URL gets past it and fails,
/// if it fails, on the network rather than on the check.
#[test]
fn an_https_url_is_not_refused_by_the_scheme_check() {
    let err = http_get("https://host.invalid./nothing-here").unwrap_err();
    assert!(!err.contains("plain HTTP"), "{err}");
}
