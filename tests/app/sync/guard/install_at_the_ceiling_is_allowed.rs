use super::*;

#[tokio::test]
async fn install_at_the_ceiling_is_allowed() {
    // The limit is inclusive: exactly `max_installs` is fine; over it is not.
    let mut cfg = config_with(20);
    cfg.guard.max_installs = 50;
    assert!(
        enforce_installs(&cfg, 50, &Reaping::new(), GuardScope::Sync)
            .await
            .is_ok()
    );
}
