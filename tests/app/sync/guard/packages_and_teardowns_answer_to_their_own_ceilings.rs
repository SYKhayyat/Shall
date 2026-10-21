use super::*;

/// **`Y20`: two ceilings, and neither spends the other's budget.** Software leaving the
/// machine and a perimeter tightening are different events. Sharing one number would make
/// the stricter govern both, so a server whose first `firewall:` declaration closes forty
/// ports could not also remove a package.
#[tokio::test]
async fn packages_and_teardowns_answer_to_their_own_ceilings() {
    let reg = Arc::new(BackendRegistry::new());
    let mut cfg = config_with(3);
    cfg.guard.max_extra_removals = 3;
    let reaping = Reaping::new();

    assert!(enforce(
        &cfg,
        &reg,
        &pairs(&["a", "b", "c"]),
        &reaping,
        GuardScope::Sync
    )
    .await
    .is_ok());
    assert!(
        enforce_extras(
            &cfg,
            &reg,
            &extras(&["link:/a", "link:/b", "link:/c"]),
            &reaping,
            GuardScope::Sync
        )
        .await
        .is_ok(),
        "three packages must not have spent the teardown budget"
    );
    assert_eq!(reaping.so_far(RemovalKind::Package), 3);
    assert_eq!(reaping.so_far(RemovalKind::Extra), 3);

    // And the package ceiling is still the package ceiling.
    let err = enforce(&cfg, &reg, &pairs(&["d"]), &reaping, GuardScope::Sync)
        .await
        .expect_err("a fourth package is over a limit of three");
    assert!(err.to_string().contains("max_removals"), "{err}");
    assert!(
        !err.to_string().contains("max_extra_removals"),
        "a package refusal must not send the reader to the teardown setting: {err}"
    );
}
