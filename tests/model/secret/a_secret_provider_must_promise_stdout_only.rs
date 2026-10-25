use super::*;

#[test]
fn a_secret_provider_must_promise_stdout_only() {
    // The T-series rule as a load-time gate: a provider that has not declared stdout_only is
    // refused, not trusted — the unsafe reading is never the default (V.80).
    let no_promise = provider("vault", &["vault", "kv", "get", "{ref}"], false);
    assert!(no_promise.unusable().unwrap().contains("stdout_only"));
    // With the promise, and a command, it loads.
    assert!(provider("vault", &["vault", "kv", "get", "{ref}"], true)
        .unusable()
        .is_none());
}
