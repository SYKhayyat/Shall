use super::*;

/// Zero is off, and off is the default. A machine that never asked for a total must not
/// start refusing the sync it ran yesterday.
#[tokio::test]
async fn the_total_is_off_by_default_and_when_zero() {
    assert_eq!(Config::default().guard.max_total_changes, 0);
    let reg = Arc::new(BackendRegistry::new());
    let cfg = config_with(0);
    let reaping = Reaping::new();
    let many: Vec<String> = (0..500).map(|i| format!("pkg{}", i)).collect();
    let refs: Vec<&str> = many.iter().map(|s| s.as_str()).collect();
    assert!(
        enforce(&cfg, &reg, &pairs(&refs), &reaping, GuardScope::Sync)
            .await
            .is_ok()
    );
    assert!(enforce_installs(&cfg, 5_000, &reaping, GuardScope::Sync)
        .await
        .is_ok());
    assert!(enforce_additions(&cfg, 5_000, &reaping, GuardScope::Sync)
        .await
        .is_ok());
}
