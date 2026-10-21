use super::*;

/// The three removal budgets are three budgets. Nineteen of each passes; the count that
/// objects is the total, and only when one is set.
#[tokio::test]
async fn each_removal_kind_spends_only_its_own_budget() {
    let reg = Arc::new(BackendRegistry::new());
    let cfg = config_with(3);
    let reaping = Reaping::new();

    assert!(enforce(
        &cfg,
        &reg,
        &pairs(&["a", "b", "c"]),
        &reaping,
        GuardScope::Sync
    )
    .await
    .is_ok());
    assert!(
        enforce_extras(
            &cfg,
            &reg,
            &extras(&["link:/a", "link:/b", "link:/c"]),
            &reaping,
            GuardScope::Sync
        )
        .await
        .is_ok(),
        "packages must not have spent the teardown budget"
    );
    assert!(
        enforce_ports(
            &cfg,
            &reg,
            &ports(&["22/tcp", "80/tcp", "443/tcp"]),
            &reaping,
            GuardScope::Sync
        )
        .await
        .is_ok(),
        "neither of the first two may have spent the port budget"
    );
    assert_eq!(reaping.so_far(RemovalKind::Package), 3);
    assert_eq!(reaping.so_far(RemovalKind::Extra), 3);
    assert_eq!(reaping.so_far(RemovalKind::Port), 3);
    assert_eq!(reaping.changes_so_far(), 9);
}
