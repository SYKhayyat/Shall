use super::*;

/// Either flag answers the total, because a total is made of both. Neither answers a
/// protection, and `--allow-mass-install` does not answer a removal count.
#[tokio::test]
async fn both_mass_flags_answer_the_total_and_only_the_total() {
    let reg = Arc::new(BackendRegistry::new());
    let mut cfg = config_with(2);
    cfg.guard.max_total_changes = 2;
    cfg.allow_mass_install = true;

    assert!(
        enforce_installs(&cfg, 50, &Reaping::new(), GuardScope::Sync)
            .await
            .is_ok(),
        "--allow-mass-install must answer the total"
    );
    let err = enforce(
        &cfg,
        &reg,
        &pairs(&["a", "b", "c"]),
        &Reaping::new(),
        GuardScope::Sync,
    )
    .await
    .expect_err("the install flag must not answer a removal count");
    assert!(err.to_string().contains("max_removals"), "{err}");
    assert!(
        !err.to_string().contains("max_total_changes"),
        "the total it *did* answer must not still be in the refusal: {err}"
    );

    cfg.allow_mass_install = false;
    cfg.allow_mass_removal = true;
    assert!(
        enforce(
            &cfg,
            &reg,
            &pairs(&["a", "b", "c"]),
            &Reaping::new(),
            GuardScope::Sync
        )
        .await
        .is_ok(),
        "--allow-mass-removal answers both counts at once"
    );
}
