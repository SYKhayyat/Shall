use super::*;

/// And the entry a *previous* run left open is not touched.
///
/// **This is the half that makes the fix safe rather than merely quiet.** An entry open
/// from an earlier run is the record that a process died holding it, and it is the only
/// thing that tells `heal` to look. A close-everything-still-open would have erased
/// exactly the state this log exists to keep, and the harness assertion it was written to
/// satisfy would have gone green either way. This run's own stranded entries close as
/// Abandoned beside it; only the ghost stays InProgress.
#[tokio::test]
async fn an_earlier_runs_open_entry_survives_this_runs_failure() {
    let mut graph = StableDiGraph::new();
    graph.add_node(GraphAction::Install(spec("gem", "logger")));
    for name in ["jq", "ripgrep"] {
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

    // A crash, before this run starts: opened and never closed by anybody.
    let ghost = h
        .journal
        .lock()
        .await
        .record_start(JournalAction::Install(spec("apt", "left-by-a-crash")))
        .unwrap();

    assert!(h.tx.execute().await.is_err());

    let j = h.journal.lock().await;
    let interrupted = j.interrupted_actions();
    assert!(
        interrupted
            .iter()
            .any(|e| e.id == ghost && e.status == ActionStatus::InProgress),
        "the earlier run's entry, still open exactly as the crash left it"
    );
    assert!(
        interrupted
            .iter()
            .filter(|e| e.id != ghost)
            .all(|e| e.status == ActionStatus::Abandoned),
        "this run's own entries are abandoned, not left reading as crashes"
    );
}
