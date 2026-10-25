use super::*;

/// **Two declared packages of one manager have no edge between them, so they are one
/// command** (II.19). They did, whenever one happened to depend on the other — which for a
/// system manager is most of a real config, and it is the case `rebuild --backend apt`
/// maximises. Measured on Ubuntu: eight packages one at a time took 31,901 ms against
/// 3,161 ms as one command.
#[tokio::test]
async fn a_native_dependency_between_declared_packages_does_not_split_the_command() {
    let backend = DepAnswering::new(&[("nginx", &["libfoo"])]);
    let registry = dep_registry(backend);
    let config = Config::default();
    let state = StateRegistry::new(PathBuf::from("test-state.json"));
    let desired: HashMap<String, Vec<PackageSpec>> = [(
        "deptest".to_string(),
        vec![declared("nginx"), declared("libfoo")],
    )]
    .into_iter()
    .collect();

    let changes = ChangePlanner::new(registry, &state, &config)
        .plan(&desired, PlanScope::Whole(HostBackends::default()))
        .await
        .unwrap();

    assert_eq!(changes.graph.node_count(), 2);
    assert_eq!(
        changes.graph.edge_count(),
        0,
        "nobody wrote `@requires`, so nothing may split the batch"
    );
}
