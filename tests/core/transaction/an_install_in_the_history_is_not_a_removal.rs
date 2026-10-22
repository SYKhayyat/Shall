use super::*;

/// Installs are not this method's business â€” its callers summarise removals.
#[tokio::test]
async fn an_install_in_the_history_is_not_a_removal() {
    let mut graph = StableDiGraph::new();
    let idx = graph.add_node(GraphAction::Install(shall::core::PackageSpec {
        name: "htop".into(),
        backend: "apt".into(),
        options: Default::default(),
        requires: vec![],
        present: true,
    }));
    let tx = tx_with(graph, vec![(idx, Prior::Absent)], Some(&[])).await;
    assert!(tx.executed_removals().is_empty());
}
