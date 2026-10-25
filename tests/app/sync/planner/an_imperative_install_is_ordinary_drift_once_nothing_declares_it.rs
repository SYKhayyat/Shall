use super::*;

#[tokio::test]
async fn an_imperative_install_is_ordinary_drift_once_nothing_declares_it() {
    // `protect_imperative` existed because an imperative install had no line, so it
    // read as drift the moment it was recorded. It has a line now
    // (`modules/imperative.txt`), so it is declared like everything else — and if that
    // line is gone, so is the reason to keep the package (II.17).
    let registry = registry_reporting("generic-test", &[]);
    let config = Config::default();
    let mut state = StateRegistry::new(PathBuf::from("test-state.json"));
    let mut imp = managed("my-imperative-tool", "generic-test");
    imp.source = "imperative".into();
    state.set_managed([imp]);
    let desired: HashMap<String, Vec<PackageSpec>> = HashMap::new();

    let changes = ChangePlanner::new(registry.clone(), &state, &config)
        .plan(&desired, PlanScope::Whole(HostBackends::default()))
        .await
        .unwrap();
    assert_eq!(changes.total_remove(), 1);
    assert_eq!(changes.removals_only().total_remove(), 1);
}
