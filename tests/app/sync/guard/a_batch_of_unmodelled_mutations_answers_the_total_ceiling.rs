use super::*;

/// A batch of mutations the model cannot name â€” `@undo=` shell commands â€” answers the
/// same total ceiling everything else answers, and what it is allowed to run is recorded
/// so whatever runs behind it counts it too.
#[tokio::test]
async fn a_batch_of_unmodelled_mutations_answers_the_total_ceiling() {
    let mut cfg = config_with(0);
    cfg.guard.max_total_changes = 3;
    let ledger = Reaping::new();

    charge_unmodelled(&cfg, &ledger, 2, GuardScope::Apply)
        .expect("two undos under a total of three");
    assert_eq!(
        ledger.changes_so_far(),
        2,
        "charged changes are on the ledger"
    );

    let err = charge_unmodelled(&cfg, &ledger, 2, GuardScope::Apply)
        .expect_err("four against a budget of three must refuse");
    assert!(
        err.to_string().contains("max_total_changes"),
        "the refusal names the setting: {err}"
    );

    cfg.allow_mass_removal = true;
    charge_unmodelled(&cfg, &ledger, 2, GuardScope::Apply)
        .expect("the mass flag is the answer, as for any other removal family");
}
