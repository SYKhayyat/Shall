use super::*;

#[tokio::test]
async fn scoped_plan_is_non_destructive() {
    let registry = registry_reporting("generic-test", &[]);
    let config = Config::default();
    let mut state = StateRegistry::new(PathBuf::from("test-state.json"));
    // A managed package that is NOT in the (empty) desired state == drift.
    state.set_managed([managed("drift-pkg-xyz", "generic-test")]);

    let desired: HashMap<String, Vec<PackageSpec>> = HashMap::new();

    // Unscoped: drift removal IS planned.
    let unscoped = {
        let planner = ChangePlanner::new(registry.clone(), &state, &config);
        planner
            .plan(&desired, PlanScope::Whole(HostBackends::default()))
            .await
            .unwrap()
    };
    assert_eq!(
        unscoped.total_remove(),
        1,
        "unscoped sync should remove drift"
    );

    // Scoped: NO removals, regardless of drift.
    let scoped = {
        let planner = ChangePlanner::new(registry.clone(), &state, &config);
        planner
            .plan(&desired, PlanScope::Narrowed(Scope::Module("dev".into())))
            .await
            .unwrap()
    };
    assert_eq!(
        scoped.total_remove(),
        0,
        "scoped upgrade must never remove packages"
    );
}
