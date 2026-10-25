use super::*;

/// V.7: `absent:` is the one exception to "Shall only removes what it manages" —
/// because you named it. So it is scheduled even though the registry never owned it.
#[tokio::test]
async fn an_absent_declaration_is_scheduled_for_removal_even_if_unmanaged() {
    // The manager is on the machine and the package is on it: an empty registry here
    // meant "no such manager", which II.7c now answers by skipping (see
    // `a_backend_this_machine_does_not_have_plans_no_removal`).
    let registry = registry_reporting("generic-test", &["libreoffice"]);
    let config = Config::default();
    let state = StateRegistry::new(PathBuf::from("test-state.json"));
    let desired: HashMap<String, Vec<PackageSpec>> = [(
        "generic-test".to_string(),
        vec![absent_spec("libreoffice", "generic-test")],
    )]
    .into_iter()
    .collect();

    let changes = ChangePlanner::new(registry, &state, &config)
        .plan(&desired, PlanScope::Whole(HostBackends::default()))
        .await
        .unwrap();

    assert!(changes.removal_tracker.contains("generic-test:libreoffice"));
}
