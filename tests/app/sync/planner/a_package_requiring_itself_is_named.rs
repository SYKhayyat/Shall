use super::*;

#[test]
fn a_package_requiring_itself_is_named() {
    let mut graph: StableDiGraph<GraphAction, ()> = StableDiGraph::new();
    let n = graph.add_node(GraphAction::Remove {
        name: "loop".into(),
        backend: "apt".into(),
    });
    graph.add_edge(n, n, ());
    // The one-element case, in the same shape as every other loop.
    let msg = describe_cycle(&graph);
    assert!(msg.contains("apt:loop requires apt:loop"), "{}", msg);
    assert!(msg.trim_end().ends_with("^ back to apt:loop"), "{}", msg);
}
