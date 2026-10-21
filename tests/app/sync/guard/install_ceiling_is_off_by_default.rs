use super::*;

#[tokio::test]
async fn install_ceiling_is_off_by_default() {
    // max_installs defaults to 0 (unset). Installs are additive and far less dangerous
    // than removals, so the ceiling stays off until a user asks for it.
    let cfg = config_with(20); // max_installs is 0 here
    assert!(
        enforce_installs(&cfg, 10_000, &Reaping::new(), GuardScope::Sync)
            .await
            .is_ok()
    );
}
