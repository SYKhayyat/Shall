use super::*;

#[tokio::test]
async fn a_command_line_is_bounded() {
    // Windows caps a command line at 8191 characters, and every manager has some limit.
    let mut graph = StableDiGraph::new();
    for i in 0..(Transaction::MAX_BATCH + 40) {
        graph.add_node(GraphAction::Install(spec("apt", &format!("pkg{}", i))));
    }
    let mut h = harness(graph, &["apt"]).await;
    h.tx.execute_with_telemetry().await.unwrap();

    assert_eq!(h.counters[0].calls.load(Ordering::SeqCst), 2);
    assert_eq!(
        h.counters[0].widest.load(Ordering::SeqCst),
        Transaction::MAX_BATCH
    );
}
