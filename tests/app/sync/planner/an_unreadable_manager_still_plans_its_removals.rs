use super::*;

/// **The removal half is unchanged, and it is the half where guessing deletes software.**
/// A manager that cannot be listed still has its removals scheduled: `is_installed` reads an
/// unreadable manager as *present*, so the plan removes and lets the removal report its own
/// failure. The other direction — reading "could not ask" as "nothing to remove" — is the one
/// that quietly leaves software installed.
#[tokio::test]
async fn an_unreadable_manager_still_plans_its_removals() {
    let unreadable = Unreadable::new("brew", usize::MAX);
    let mut registry = BackendRegistry::new();
    registry.register(Arc::new(
        shall::core::manager::BackendCapabilities::builder(unreadable.core())
            .with_queryable(unreadable.clone())
            .build(),
    ));
    let state = StateRegistry::new(PathBuf::from("test-state.json"));
    let desired: HashMap<String, Vec<PackageSpec>> =
        [("brew".to_string(), vec![absent_spec("libreoffice", "brew")])]
            .into_iter()
            .collect();

    let changes = ChangePlanner::new(Arc::new(registry), &state, &Config::default())
        .plan(&desired, PlanScope::Whole(HostBackends::default()))
        .await
        .expect("an unreadable manager must not fail the plan")
        .total_remove();
    assert_eq!(changes, 1, "not knowing must never mean not removing");
}
