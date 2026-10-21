use super::*;

#[tokio::test]
async fn yes_does_not_override_the_install_ceiling() {
    // -y is what every script passes; it must not green-light a manifest-globbed flood.
    let mut cfg = config_with(20);
    cfg.guard.max_installs = 50;
    cfg.yes = true;
    assert!(
        enforce_installs(&cfg, 5_000, &Reaping::new(), GuardScope::Sync)
            .await
            .is_err()
    );
}
