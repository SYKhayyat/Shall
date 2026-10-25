use super::*;

/// The reason `adopt` made every later sync fail: a `service:` line was `unverifiable`, and
/// unverifiable places — so 150 adopted running services meant 150 `sc start` calls per
/// sync, each on a service that was already running.
#[tokio::test]
async fn a_service_already_in_its_declared_state_is_not_placed_again() {
    let config = Arc::new(shall::config::Config::default());
    let reg = registry_listing_nginx();
    let exec = CommandExecutor::new(false, false);

    let (running, key) = service("nginx", "status", "running");
    assert_eq!(
        in_effect(&config, &reg, &exec, &running, &key).await,
        Some(true),
        "nginx is in the listing and the line asks for running"
    );

    let (stopped, key) = service("nginx", "status", "stopped");
    assert_eq!(
        in_effect(&config, &reg, &exec, &stopped, &key).await,
        Some(false),
        "a running service declared stopped is drift, and drift places"
    );

    // The same two questions about a service the init does not report.
    let (running, key) = service("absent-svc", "status", "running");
    assert_eq!(
        in_effect(&config, &reg, &exec, &running, &key).await,
        Some(false)
    );
    let (stopped, key) = service("absent-svc", "status", "stopped");
    assert_eq!(
        in_effect(&config, &reg, &exec, &stopped, &key).await,
        Some(true)
    );
}
