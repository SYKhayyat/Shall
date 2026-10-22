use super::*;

/// A run that outlives its deadline closes the entries it opened **as abandoned**.
///
/// The timeout drops the whole `JoinSet`, so the batches are killed inside the manager's
/// command and reach neither of the calls that close an entry. What those entries may not
/// be closed as is *Failed*: a failed attempt reached an outcome (Q33), these reached
/// nothing, and only the interrupted set — InProgress and Abandoned — is what `heal`
/// finishes. An install killed mid-command may have half-run; that is exactly what heal
/// exists for.
#[tokio::test]
async fn a_run_that_times_out_closes_its_entries_as_abandoned() {
    let mut graph = StableDiGraph::new();
    for name in ["jq", "ripgrep", "fd", "bat"] {
        graph.add_node(GraphAction::Install(spec("apt", name)));
    }
    // Each call takes far longer than the whole run is allowed, so the deadline lands
    // while work is outstanding — which is the only way to reach the cancellation arm.
    let mut h = harness_with(
        graph,
        &["apt"],
        Duration::from_secs(30),
        Duration::from_millis(150),
        &[],
    )
    .await;

    let outcome = h.tx.execute().await;
    assert!(outcome.is_err(), "the deadline must end the run");

    let j = h.journal.lock().await;
    let interrupted = j.interrupted_actions();
    let in_progress: Vec<_> = interrupted
        .iter()
        .filter(|e| e.status == ActionStatus::InProgress)
        .collect();
    assert!(
        in_progress.is_empty(),
        "nothing of this run's may read as a crash it did not have: {:?}",
        in_progress
            .iter()
            .map(|e| e.action.key())
            .collect::<Vec<_>>()
    );
    let abandoned = interrupted
        .iter()
        .filter(|e| e.status == ActionStatus::Abandoned)
        .count();
    assert!(
        abandoned > 0,
        "the run opened work that never got an answer, and heal must see it"
    );
}
