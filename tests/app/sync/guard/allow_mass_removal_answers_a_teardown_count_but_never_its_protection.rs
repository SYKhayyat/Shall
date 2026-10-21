use super::*;

#[tokio::test]
async fn allow_mass_removal_answers_a_teardown_count_but_never_its_protection() {
    // The extras half of `allow_mass_removal_answers_the_count_and_nothing_else`, and for
    // the same reason: the flag means "yes, that many is what I meant", never "yes, delete
    // the one I told you to keep".
    let reg = Arc::new(BackendRegistry::new());
    let mut cfg = config_with(1);
    cfg.guard.protected_packages = vec!["keep".into()];
    cfg.allow_mass_removal = true;

    assert!(
        enforce_extras(
            &cfg,
            &reg,
            &extras(&["link:/a", "link:/b"]),
            &Reaping::new(),
            GuardScope::Sync
        )
        .await
        .is_ok(),
        "the flag must let a big-but-ordinary teardown through"
    );
    assert!(
        enforce_extras(
            &cfg,
            &reg,
            &extras(&["link:/keep"]),
            &Reaping::new(),
            GuardScope::Sync
        )
        .await
        .is_err(),
        "nothing overrides protection â€” not even --allow-mass-removal"
    );
}
