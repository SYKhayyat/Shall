use super::*;

#[tokio::test]
async fn sync_removes_what_it_manages_and_you_stopped_declaring() {
    // V.34: sync removes drift BY DEFINITION. `prune_on_sync` made that a setting, so
    // sync could be configured into something that is not sync — and a separate prune command was
    // sync with the install half amputated.
    let registry = registry_reporting("generic-test", &[]);
    let config = Config::default();
    let mut state = StateRegistry::new(PathBuf::from("test-state.json"));
    state.set_managed([managed("drift-pkg-xyz", "generic-test")]);
    let desired: HashMap<String, Vec<PackageSpec>> = HashMap::new();

    let changes = ChangePlanner::new(registry.clone(), &state, &config)
        .plan(&desired, PlanScope::Whole(HostBackends::default()))
        .await
        .unwrap();
    assert_eq!(
        changes.total_remove(),
        1,
        "a managed package nothing declares is drift, and removing it is what sync is"
    );
}
