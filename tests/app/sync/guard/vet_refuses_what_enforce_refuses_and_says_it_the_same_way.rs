use super::*;

/// One rule, one sentence: what the prompt-time ask refuses it refuses in the same words
/// the engine would use, so a preview never disagrees with the verdict it foretold.
#[tokio::test]
async fn vet_refuses_what_enforce_refuses_and_says_it_the_same_way() {
    let reg = Arc::new(BackendRegistry::new());
    let cfg = config_with(2);
    let many = pairs(&["jq", "htop", "bat"]);

    let asked = vet(
        &cfg,
        &reg,
        &many,
        RemovalKind::Package,
        &Reaping::new(),
        GuardScope::Sync,
    )
    .await
    .expect_err("three removals over a ceiling of two refuse")
    .to_string();
    let enforced = enforce(&cfg, &reg, &many, &Reaping::new(), GuardScope::Sync)
        .await
        .expect_err("enforce refuses the same set")
        .to_string();
    assert_eq!(asked, enforced);
}
