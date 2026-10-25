use super::*;

/// A scoped run is non-destructive, and that must hold for `absent:` too.
#[tokio::test]
async fn a_scoped_run_does_not_act_on_absent_declarations() {
    let registry = Arc::new(BackendRegistry::new());
    let config = Config::default();
    let state = StateRegistry::new(PathBuf::from("test-state.json"));
    let desired: HashMap<String, Vec<PackageSpec>> = [(
        "generic-test".to_string(),
        vec![absent_spec("libreoffice", "generic-test")],
    )]
    .into_iter()
    .collect();

    let changes = ChangePlanner::new(registry, &state, &config)
        .plan(&desired, PlanScope::Narrowed(Scope::Module("dev".into())))
        .await
        .unwrap();

    assert_eq!(changes.total_remove(), 0);
    assert_eq!(changes.total_install(), 0);
}
