use super::*;

/// …and a `@requires` the user *did* write still orders the two sides (`Y1`).
#[tokio::test]
async fn a_written_requires_is_still_an_edge() {
    let backend = DepAnswering::new(&[]);
    let registry = dep_registry(backend);
    let config = Config::default();
    let state = StateRegistry::new(PathBuf::from("test-state.json"));
    let mut dependent = declared("nginx");
    dependent.requires = vec!["deptest:libfoo".to_string()];
    let desired: HashMap<String, Vec<PackageSpec>> =
        [("deptest".to_string(), vec![dependent, declared("libfoo")])]
            .into_iter()
            .collect();

    let changes = ChangePlanner::new(registry, &state, &config)
        .plan(&desired, PlanScope::Whole(HostBackends::default()))
        .await
        .unwrap();

    assert_eq!(changes.graph.edge_count(), 1);
}
