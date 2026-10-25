use super::*;

/// `absent:` shares the desired-state map with wishes, because the map type is the
/// seam. Everything downstream of `plan` reads that map as a wish list, so an absent
/// declaration that survives into it gets INSTALLED — the exact opposite of what the
/// line says.
#[tokio::test]
async fn an_absent_declaration_is_never_installed() {
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
        .plan(&desired, PlanScope::Whole(HostBackends::default()))
        .await
        .unwrap();

    assert_eq!(
        changes.total_install(),
        0,
        "an `absent:` line must never become an install"
    );
}
