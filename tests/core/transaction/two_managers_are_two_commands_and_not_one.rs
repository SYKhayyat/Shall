use super::*;

#[tokio::test]
async fn two_managers_are_two_commands_and_not_one() {
    let mut graph = StableDiGraph::new();
    graph.add_node(GraphAction::Install(spec("apt", "jq")));
    graph.add_node(GraphAction::Install(spec("apt", "ripgrep")));
    graph.add_node(GraphAction::Install(spec("npm", "prettier")));
    let mut h = harness(graph, &["apt", "npm"]).await;
    h.tx.execute_with_telemetry().await.unwrap();

    assert_eq!(h.counters[0].calls.load(Ordering::SeqCst), 1);
    assert_eq!(h.counters[0].widest.load(Ordering::SeqCst), 2);
    assert_eq!(h.counters[1].calls.load(Ordering::SeqCst), 1);
    assert_eq!(h.counters[1].widest.load(Ordering::SeqCst), 1);
}
