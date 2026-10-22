use super::*;

/// A removal whose target was never there changed nothing, and counting it would
/// inflate the summary with work the machine never noticed.
#[tokio::test]
async fn a_removal_of_something_absent_is_not_counted_as_gone() {
    let mut graph = StableDiGraph::new();
    let jq = remove_node(&mut graph, "apt", "jq");
    let tx = tx_with(graph, vec![(jq, Prior::Absent)], Some(&[])).await;
    assert!(tx.executed_removals().is_empty());
}
