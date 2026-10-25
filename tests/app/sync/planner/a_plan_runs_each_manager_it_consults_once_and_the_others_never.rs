use super::*;

/// Each manager the plan consults is run **once**, and no manager it does not consult is
/// run at all.
///
/// The first half is what makes the plan's fan-out worth having: the fan-out is over specs,
/// so twenty declarations of one backend are twenty futures waiting on one listing, and if
/// that listing were fetched per spec the wave would be twenty subprocesses deep. The
/// second half is the guard on the fix for it — asking every manager up front is right only
/// while "every manager" means the ones this plan was going to ask anyway. Widening it to
/// the registry would wake a dozen managers for a one-line manifest, which is the cost this
/// was supposed to remove, spent on a different run.
#[tokio::test]
async fn a_plan_runs_each_manager_it_consults_once_and_the_others_never() {
    let mut registry = BackendRegistry::new();
    let declared = register_fake(&mut registry, "declared-mgr", &["jq", "curl"]);
    let bystander = register_fake(&mut registry, "bystander-mgr", &["vim"]);
    let registry = Arc::new(registry);

    let specs: Vec<PackageSpec> = ["jq", "curl", "ripgrep", "fd"]
        .iter()
        .map(|n| PackageSpec {
            name: (*n).to_string(),
            backend: "declared-mgr".to_string(),
            ..Default::default()
        })
        .collect();
    let desired: HashMap<String, Vec<PackageSpec>> =
        [("declared-mgr".to_string(), specs)].into_iter().collect();

    let config = Config::default();
    let state = StateRegistry::new(PathBuf::from("test-state.json"));
    let changes = ChangePlanner::new(registry, &state, &config)
        .plan(&desired, PlanScope::Whole(HostBackends::default()))
        .await
        .unwrap();

    use std::sync::atomic::Ordering;
    assert_eq!(
        declared.load(Ordering::SeqCst),
        1,
        "four declarations asked its manager more than once"
    );
    assert_eq!(
        bystander.load(Ordering::SeqCst),
        0,
        "a manager nothing declares was woken by a plan that had no question for it"
    );
    // And the plan itself is unchanged by any of that: two installed, two to install.
    assert_eq!(changes.total_install(), 2);
}
