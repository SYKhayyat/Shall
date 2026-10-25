use super::*;

/// **The bug: one manager that could not answer cost the whole machine its sync.** The other
/// manager's declaration is planned, applied and reported; the unreadable one is named and
/// left alone. Before this, the `?` out of the fan-out ended the plan on the first failure —
/// the command exited 1 having applied nothing, over a manager that was merely having a bad
/// moment.
#[tokio::test]
async fn a_manager_that_cannot_answer_costs_only_its_own_declarations() {
    let config = Config::default();
    let unreadable = Unreadable::new("brew", usize::MAX);
    let registry = registry_with_unreadable(&unreadable, &[]);

    let changes = plan_with(
        registry,
        &config,
        &["generic-test", "brew"],
        &[("generic-test", "jq"), ("brew", "wget")],
    )
    .await;

    assert_eq!(
        changes.total_install(),
        1,
        "the manager that answered must still be planned"
    );
    let unanswerable: Vec<&shall::app::sync::planner::Skipped> = changes
        .skipped
        .iter()
        .filter(|s| s.kind == shall::app::sync::planner::SkipKind::CouldNotAsk)
        .collect();
    assert_eq!(
        unanswerable.len(),
        1,
        "the declaration that could not be read must be named: {:?}",
        changes.skipped
    );
    assert_eq!(unanswerable[0].key, "brew:wget");
    assert!(
        unanswerable[0].reason.contains("brew"),
        "the row names the manager, not just the package: {}",
        unanswerable[0].reason
    );
    assert_eq!(unreadable.asks(), 1, "the default asks once and gives up");
}
