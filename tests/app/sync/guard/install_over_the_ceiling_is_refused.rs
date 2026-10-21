use super::*;

#[tokio::test]
async fn install_over_the_ceiling_is_refused() {
    let mut cfg = config_with(20);
    cfg.guard.max_installs = 50;
    let err = enforce_installs(&cfg, 51, &Reaping::new(), GuardScope::Sync)
        .await
        .expect_err("51 installs over a limit of 50 must be refused");
    let msg = err.to_string();
    assert!(msg.contains("installs 51 packages"), "{}", msg);
    assert!(msg.contains("max_installs"), "{}", msg);
    assert!(msg.contains("--allow-mass-install"), "{}", msg);
}
