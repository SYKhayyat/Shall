use super::*;

#[tokio::test]
async fn allow_mass_removal_answers_the_count_and_nothing_else() {
    // II.10: `--allow-mass-removal` is the answer to ONE refusal â€” the count. It used
    // to clear every objection, so the flag meaning "yes, 50 is what I meant" also
    // deleted python3. A confirmation asks; a refusal says no (V.26).
    let reg = Arc::new(BackendRegistry::new());
    let mut cfg = config_with(2);
    cfg.allow_mass_removal = true;

    // The count alone: allowed, because that is what the flag is for.
    assert!(
        enforce(
            &cfg,
            &reg,
            &pairs(&["jq", "htop", "bat"]),
            &Reaping::new(),
            GuardScope::Sync
        )
        .await
        .is_ok(),
        "the flag must let a big-but-ordinary removal through"
    );

    // A protected package, even when the flag is set and the count is fine.
    assert!(
        enforce(
            &cfg,
            &reg,
            &pairs(&["python3"]),
            &Reaping::new(),
            GuardScope::Sync
        )
        .await
        .is_err(),
        "nothing overrides protection â€” not even --allow-mass-removal"
    );

    // And a big removal that also touches a protected package is still refused.
    assert!(
        enforce(
            &cfg,
            &reg,
            &pairs(&["jq", "htop", "bat", "python3"]),
            &Reaping::new(),
            GuardScope::Sync
        )
        .await
        .is_err(),
        "the flag must not carry a protected package in on the back of the count"
    );
}
