use super::*;

/// Protection reaches a port the same way it reaches a resource: `protected_packages` names
/// the thing, and the count is not what is being asked.
#[tokio::test]
async fn a_protected_port_is_refused_whatever_the_count() {
    let reg = Arc::new(BackendRegistry::new());
    let mut cfg = config_with(20);
    cfg.guard.protected_packages = vec!["22/tcp".into()];
    cfg.allow_mass_removal = true;
    let err = enforce_ports(
        &cfg,
        &reg,
        &ports(&["22/tcp"]),
        &Reaping::new(),
        GuardScope::Sync,
    )
    .await
    .expect_err("a protected port is a refusal, not a count");
    assert!(err.to_string().contains("22/tcp"), "{err}");
}
