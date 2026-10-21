use super::*;

/// The prompt-time ask must decide without spending. `remove-orphans` and
/// `purge-undeclared` vet before their confirmation prompt and the engine enforces over
/// the same pairs after it; a vet that recorded would have that second ask measuring
/// N + N against the ceiling and refusing a set the user had already confirmed.
#[tokio::test]
async fn a_vet_decides_without_spending_the_budget() {
    let reg = Arc::new(BackendRegistry::new());
    let mut cfg = config_with(0);
    cfg.guard.max_total_changes = 10;
    let six = pairs(&["a", "b", "c", "d", "e", "f"]);
    let ledger = Reaping::new();

    vet(
        &cfg,
        &reg,
        &six,
        RemovalKind::Package,
        &ledger,
        GuardScope::RemoveOrphans,
    )
    .await
    .expect("six orphans pass the prompt-time ask");
    assert_eq!(ledger.changes_so_far(), 0, "deciding is not spending");

    // The engine's ask over the same pairs, through the same ledger: with the bug this
    // measured six spent plus six planned against a budget of ten.
    enforce(&cfg, &reg, &six, &ledger, GuardScope::RemoveOrphans)
        .await
        .expect("the confirmed set clears exactly once");
    assert_eq!(ledger.changes_so_far(), 6);

    // And the spend is real: a second identical phase answers to what the first used,
    // which is the property the record exists to keep.
    assert!(
        enforce(&cfg, &reg, &six, &ledger, GuardScope::RemoveOrphans)
            .await
            .is_err(),
        "twelve changes against a budget of ten must refuse"
    );
}
