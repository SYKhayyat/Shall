use super::*;

/// The total counts what the per-kind ceilings never look at: installs and upgrades,
/// resources written, ports opened. A total that only counted removals would be the removal
/// ceiling with a longer name.
#[tokio::test]
async fn the_total_counts_additions_as_well_as_removals() {
    let reg = Arc::new(BackendRegistry::new());
    let mut cfg = config_with(0);
    cfg.guard.max_total_changes = 10;
    let reaping = Reaping::new();

    assert!(enforce_installs(&cfg, 6, &reaping, GuardScope::Sync)
        .await
        .is_ok());
    assert!(enforce_additions(&cfg, 3, &reaping, GuardScope::Sync)
        .await
        .is_ok());
    assert_eq!(reaping.changes_so_far(), 9);

    let err = enforce(&cfg, &reg, &pairs(&["a", "b"]), &reaping, GuardScope::Sync)
        .await
        .expect_err("nine changes plus two removals is eleven, over ten");
    assert!(
        err.to_string().contains("makes 11 changes in total"),
        "{err}"
    );

    // And the additive gates answer to it in their own right.
    let err = enforce_additions(&cfg, 2, &reaping, GuardScope::Sync)
        .await
        .expect_err("nine plus two is eleven whichever gate counts it");
    assert!(err.to_string().contains("max_total_changes"), "{err}");
}
