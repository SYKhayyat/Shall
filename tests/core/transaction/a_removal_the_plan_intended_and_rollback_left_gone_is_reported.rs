use super::*;

/// The case the summary owes its numbers to: completed removals the plan itself intended,
/// which rollback left gone. A run killed after these reports them as removed.
#[tokio::test]
async fn a_removal_the_plan_intended_and_rollback_left_gone_is_reported() {
    let mut graph = StableDiGraph::new();
    let jq = remove_node(&mut graph, "apt", "jq");
    let tx = tx_with(
        graph,
        vec![(jq, Prior::Present(Some("1.6".into())))],
        Some(&[]),
    )
    .await;
    assert_eq!(
        tx.executed_removals(),
        vec![("apt".to_string(), "jq".to_string())]
    );
}
