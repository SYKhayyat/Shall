use super::*;

/// Removals reach the graph and the tracker together. The tracker is what `declined`
/// consults to answer "is this already scheduled", so a removal in one and not the other
/// is a removal that can be scheduled twice.
#[test]
fn a_frozen_removal_is_in_the_graph_and_in_the_tracker() {
    let removals = vec![PlanRemoval {
        backend: "apt".into(),
        name: "vim".into(),
    }];
    let changes = saved_plan_to_changes(&[], &removals);

    assert_eq!(changes.total_remove(), 1);
    assert!(changes.removal_tracker.contains("apt:vim"));
    assert!(matches!(
        changes.graph.node_weights().next(),
        Some(GraphAction::Remove { name, backend }) if name == "vim" && backend == "apt"
    ));
}
