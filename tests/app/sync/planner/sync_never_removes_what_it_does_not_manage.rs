use super::*;

#[tokio::test]
async fn sync_never_removes_what_it_does_not_manage() {
    // II.7: what Shall may remove is what it manages and you stopped declaring, plus
    // `absent:`. Nothing else, ever. `prune_scope = "system"` was a setting that broke
    // that rule — a routine sync deleting software it never installed (V.21). It is
    // `purge-undeclared` instead: a command you type, not a mode you inherit.
    let registry = Arc::new(BackendRegistry::new());
    let config = Config::default();
    // Nothing managed, nothing desired: an untouched machine full of software.
    let state = StateRegistry::new(PathBuf::from("test-state.json"));
    let desired: HashMap<String, Vec<PackageSpec>> = HashMap::new();

    let changes = ChangePlanner::new(registry.clone(), &state, &config)
        .plan(&desired, PlanScope::Whole(HostBackends::default()))
        .await
        .unwrap();
    assert_eq!(
        changes.total_remove(),
        0,
        "sync must never reach outside what it manages"
    );
}
