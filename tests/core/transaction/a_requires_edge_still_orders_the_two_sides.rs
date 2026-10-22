use super::*;

#[tokio::test]
async fn a_requires_edge_still_orders_the_two_sides() {
    // A batch is made of what is ready *at the same moment*, so an edge splits it â€”
    // otherwise a package would go on the same command line as the thing it requires.
    // Only a written `@requires` produces one (`Y9`); a native dependency the manager
    // resolves for itself does not, and the two used to be indistinguishable here.
    let mut graph = StableDiGraph::new();
    let first = graph.add_node(GraphAction::Install(spec("apt", "libfoo")));
    let second = graph.add_node(GraphAction::Install(spec("apt", "foo-tool")));
    graph.add_edge(first, second, ());
    let mut h = harness(graph, &["apt"]).await;
    h.tx.execute_with_telemetry().await.unwrap();

    assert_eq!(
        h.counters[0].calls.load(Ordering::SeqCst),
        2,
        "a required package and its dependent cannot go on one command line"
    );
    assert_eq!(h.counters[0].widest.load(Ordering::SeqCst), 1);
}
