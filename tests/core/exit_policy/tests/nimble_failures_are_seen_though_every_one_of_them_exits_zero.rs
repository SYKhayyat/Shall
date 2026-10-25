use super::*;

/// nimble reports every failure on a line beginning `Error:` and exits 0 regardless, so
/// without a policy Shall called a build that never produced a binary a successful
/// install — and then the harness's `list` and on-PATH checks failed, which is the
/// product telling the truth downstream about a lie it told upstream.
#[test]
fn nimble_failures_are_seen_though_every_one_of_them_exits_zero() {
    let p = nimble();
    for (case, out) in [
        ("build failed", NIMBLE_BUILD_FAILED),
        ("package not found", NIMBLE_NOT_FOUND),
        ("uninstall missing", NIMBLE_UNINSTALL_MISSING),
    ] {
        assert!(
            p.signals_failure(&ExitPolicy::haystack(out.as_bytes(), b"")),
            "nimble failure not detected: {case}"
        );
    }
}
