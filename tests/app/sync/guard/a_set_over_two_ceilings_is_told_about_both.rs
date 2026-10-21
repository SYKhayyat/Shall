use super::*;

/// A refusal that names two ceilings when two are busted. Naming one sends a user to raise
/// a number and meet the other on the next run.
#[tokio::test]
async fn a_set_over_two_ceilings_is_told_about_both() {
    let reg = Arc::new(BackendRegistry::new());
    let mut cfg = config_with(2);
    cfg.guard.max_total_changes = 2;
    let err = enforce(
        &cfg,
        &reg,
        &pairs(&["a", "b", "c"]),
        &Reaping::new(),
        GuardScope::Sync,
    )
    .await
    .expect_err("three is over both");
    let msg = err.to_string();
    assert!(msg.contains("max_removals"), "{msg}");
    assert!(msg.contains("max_total_changes"), "{msg}");
}
