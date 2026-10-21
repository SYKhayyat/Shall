use super::*;

#[tokio::test]
async fn a_deliberate_purge_ignores_the_count_but_never_protection() {
    // II.11: `max_removals` catches accidents, and `purge-undeclared` is the opposite of
    // an accident â€” you typed its name. `protected_packages` and OS-essential still
    // apply, and the ratio check is what asks whether you meant it at all.
    let reg = Arc::new(BackendRegistry::new());
    let cfg = config_with(2);

    assert!(
        enforce_deliberate(
            &cfg,
            &reg,
            &pairs(&["a", "b", "c", "d"]),
            &Reaping::new(),
            GuardScope::PurgeUndeclared
        )
        .await
        .is_ok(),
        "the count is not the question here"
    );
    assert!(
        enforce_deliberate(
            &cfg,
            &reg,
            &pairs(&["python3"]),
            &Reaping::new(),
            GuardScope::PurgeUndeclared
        )
        .await
        .is_err(),
        "protection still applies to a deliberate purge"
    );
}
