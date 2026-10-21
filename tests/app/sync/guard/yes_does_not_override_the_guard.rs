use super::*;

#[tokio::test]
async fn yes_does_not_override_the_guard() {
    // The whole point: -y is what every script passes. It must not mean "purge".
    let reg = Arc::new(BackendRegistry::new());
    let mut cfg = config_with(20);
    cfg.yes = true;
    assert!(enforce(
        &cfg,
        &reg,
        &pairs(&["python3"]),
        &Reaping::new(),
        GuardScope::Apply
    )
    .await
    .is_err());
}
