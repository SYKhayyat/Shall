use super::*;

#[tokio::test]
async fn six_independent_installs_are_one_command() {
    // Measured before this: six declared packages produced six separate apt processes and
    // 12,465 ms, against 3,161 ms for the same packages as one command. The batching code
    // in `generic::install_group` was already written and had never been handed more than
    // one spec.
    let mut graph = StableDiGraph::new();
    for name in ["lolcat", "cowsay", "pv", "sl", "toilet", "cmatrix"] {
        graph.add_node(GraphAction::Install(spec("apt", name)));
    }
    let mut h = harness(graph, &["apt"]).await;
    let results = h.tx.execute_with_telemetry().await.unwrap();

    assert_eq!(
        h.counters[0].calls.load(Ordering::SeqCst),
        1,
        "six packages, one manager, no edges between them â€” that is one command"
    );
    assert_eq!(h.counters[0].widest.load(Ordering::SeqCst), 6);
    assert_eq!(results.len(), 6, "every package still gets its own result");
    assert!(
        results.iter().all(|r| r.batch_size == 6),
        "the telemetry has to say why six durations are identical"
    );
}
