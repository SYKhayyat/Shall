use super::*;

/// **A ceiling is a budget for the command.** A sync tears extras down in two places â€” the
/// firewall's undeclared ports and the ledger's drift â€” and each used to check only its own
/// list, so a limit of five could be passed twice by a run that exceeded it once.
///
/// The number is no longer a parameter. `Reaping` carries it, and this test drives the real
/// one through two `enforce_extras` calls the way a sync does.
#[tokio::test]
async fn two_teardown_phases_of_one_command_share_one_budget() {
    let reg = Arc::new(BackendRegistry::new());
    let cfg = config_with(5);
    let reaping = Reaping::new();

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
        "three teardowns under a limit of five must pass on their own"
    );
    assert_eq!(reaping.so_far(RemovalKind::Extra), 3);

    let err = enforce_extras(
        &cfg,
        &reg,
        &extras(&["link:/d", "link:/e", "link:/f"]),
        &reaping,
        GuardScope::Sync,
    )
    .await
    .expect_err("three more, after three, is six over a limit of five");
    assert!(err.to_string().contains("6 managed resources"), "{err}");
    assert!(
        err.to_string().contains("max_extra_removals"),
        "the refusal has to name the setting the reader must change: {err}"
    );
    assert_eq!(
        reaping.so_far(RemovalKind::Extra),
        3,
        "a refused set must not raise the total anything behind it is measured against"
    );
}
