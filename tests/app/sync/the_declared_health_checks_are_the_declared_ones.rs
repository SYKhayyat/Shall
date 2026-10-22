use super::*;

/// **The declared health checks are the ones that were declared.**
///
/// Found by the mutation gate: `declared_health_checks` replaced with `vec![]` survived the
/// whole suite. **It is where the machine-wide list and the per-package list are joined**, and
/// an empty list means *no health checks at all* — so the two checks that read it
/// (`require_revert_path` and `require_health_commands_approved`) have nothing to refuse over
/// and nothing to ask the ledger about.
///
/// **Both halves, because U7 ruled they are not alternatives**: the code that decides whether
/// the machine is healthy must never be able to consult one and forget the other — which is
/// what two collection sites would eventually mean.
#[tokio::test]
async fn the_declared_health_checks_are_the_declared_ones() {
    let (engine, _mock) = engine(
        "declared-checks",
        config_with_health("declared-checks", &["port:8080", "port:9090"]),
    )
    .await;

    let checks = engine.declared_health_checks(&no_changes());
    assert_eq!(
        checks.len(),
        2,
        "both of the machine-wide checks must be collected, and nothing else: {checks:?}"
    );
    assert!(
        checks.iter().all(|c| c.subject == "preferences.toml"),
        "the machine-wide checks are the ones the preferences declared: {checks:?}"
    );
}
