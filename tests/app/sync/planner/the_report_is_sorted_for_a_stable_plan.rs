use super::*;

/// The plan is displayed in a stable, sorted order regardless of how the graph was built
/// — the node order follows dependency edges and a HashMap crawl, so without the sort in
/// `generate_report` the same change set printed differently each run.
#[test]
fn the_report_is_sorted_for_a_stable_plan() {
    use petgraph::stable_graph::StableDiGraph;
    let ins = |name: &str, backend: &str| {
        GraphAction::Install(PackageSpec {
            name: name.into(),
            backend: backend.into(),
            options: Default::default(),
            requires: vec![],
            present: true,
        })
    };
    let mut graph: StableDiGraph<GraphAction, ()> = StableDiGraph::new();
    // Add out of order, across backends.
    graph.add_node(ins("zsh", "apt"));
    graph.add_node(ins("bat", "cargo"));
    graph.add_node(ins("acl", "apt"));
    graph.add_node(GraphAction::Remove {
        name: "nano".into(),
        backend: "apt".into(),
    });
    graph.add_node(GraphAction::Remove {
        name: "amp".into(),
        backend: "cargo".into(),
    });
    let changes = SyncChanges {
        graph,
        ..Default::default()
    };

    let report = changes.generate_report();
    let installs: Vec<(&str, &str)> = report
        .install
        .iter()
        .map(|e| (e.backend.as_str(), e.name.as_str()))
        .collect();
    assert_eq!(
        installs,
        vec![("apt", "acl"), ("apt", "zsh"), ("cargo", "bat")]
    );
    let removes: Vec<(&str, &str)> = report
        .remove
        .iter()
        .map(|e| (e.backend.as_str(), e.name.as_str()))
        .collect();
    assert_eq!(removes, vec![("apt", "nano"), ("cargo", "amp")]);
}
