use super::*;

/// The macOS nightly's own shape: one manager fails while another is mid-command.
///
/// This is the path that actually stranded 22 operations. `continue_past` is `Nothing`, so
/// the first failure ends the run — and every batch still inside a manager's command is
/// killed where it stands, having opened its WAL entries and closed none. Closed as
/// Abandoned, so `heal` finishes them rather than walking past on the Q33 rule.
#[tokio::test]
async fn a_run_stopped_by_one_managers_failure_closes_its_entries_as_abandoned() {
    let mut graph = StableDiGraph::new();
    // The one that fails, and three the slow manager is still working through when it does.
    graph.add_node(GraphAction::Install(spec("gem", "logger")));
    for name in ["jq", "ripgrep", "fd"] {
        graph.add_node(GraphAction::Install(spec("apt", name)));
    }
    let mut h = harness_with(
        graph,
        &["apt", "gem"],
        Duration::from_secs(30),
        Duration::from_secs(3600),
        &["gem"],
    )
    .await;

    assert!(h.tx.execute().await.is_err(), "the failing manager ends it");

    let j = h.journal.lock().await;
    let statuses: Vec<_> = j
        .interrupted_actions()
        .iter()
        .map(|e| (e.action.key(), e.status))
        .collect();
    assert!(
        statuses.iter().all(|(_, s)| *s == ActionStatus::Abandoned),
        "every stranded entry is abandoned, which is what heal finishes: {statuses:?}"
    );
}
