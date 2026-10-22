use super::*;

/// A removal rollback reinstated must not be reported as gone â€” the machine has the
/// package back, whatever the history says.
#[tokio::test]
async fn a_removal_that_was_not_intended_is_treated_as_reinstated() {
    let mut graph = StableDiGraph::new();
    let jq = remove_node(&mut graph, "apt", "jq");
    // Declared still carries `apt:jq`, so rollback put it back (`U41`).
    let tx = tx_with(
        graph,
        vec![(jq, Prior::Present(Some("1.6".into())))],
        Some(&["apt:jq"]),
    )
    .await;
    assert!(tx.executed_removals().is_empty());
}
