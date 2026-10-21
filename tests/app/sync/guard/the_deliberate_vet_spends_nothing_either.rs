use super::*;

/// The deliberate twin: `purge-undeclared`'s prompt-time ask ignores the counts and must
/// also write nothing, or its engine ask measures double against `max_total_changes`.
#[tokio::test]
async fn the_deliberate_vet_spends_nothing_either() {
    let reg = Arc::new(BackendRegistry::new());
    let mut cfg = config_with(0);
    cfg.guard.max_total_changes = 8;
    let six = pairs(&["a", "b", "c", "d", "e", "f"]);
    let ledger = Reaping::new();

    vet_deliberate(&cfg, &reg, &six, GuardScope::PurgeUndeclared)
        .await
        .expect("protection passes, and the counts are not the question");
    assert_eq!(
        ledger.changes_so_far(),
        0,
        "the prompt-time ask writes nothing"
    );

    enforce_deliberate(&cfg, &reg, &six, &ledger, GuardScope::PurgeUndeclared)
        .await
        .expect("the engine's ask measures six, not twelve");
    assert_eq!(
        ledger.changes_so_far(),
        6,
        "the engine's ask is the one that spends"
    );
}
