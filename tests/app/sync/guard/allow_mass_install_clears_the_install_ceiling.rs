use super::*;

#[tokio::test]
async fn allow_mass_install_clears_the_install_ceiling() {
    // Symmetric to --allow-mass-removal answering the removal count.
    let mut cfg = config_with(20);
    cfg.guard.max_installs = 50;
    cfg.allow_mass_install = true;
    assert!(
        enforce_installs(&cfg, 5_000, &Reaping::new(), GuardScope::Sync)
            .await
            .is_ok()
    );
}
