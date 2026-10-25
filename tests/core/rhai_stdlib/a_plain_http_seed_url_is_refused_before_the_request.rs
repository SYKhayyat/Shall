use super::*;

/// **The URL a script hands in is checked, not only the ones it is redirected to.**
///
/// The pooled client refuses a hop that leaves HTTPS, so the seed URL was the one place its
/// policy never looked. What `http_get` returns becomes a resolved variable, and a resolved
/// variable decides which packages get declared — the value crossing the network decides
/// what runs on the machine, which is `check_scheme`'s whole argument.
#[test]
fn a_plain_http_seed_url_is_refused_before_the_request() {
    let err = http_get("http://internal/config").expect_err("plain http is not a seed URL");
    assert!(err.contains("plain HTTP"), "{err}");
}
