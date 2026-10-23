use super::*;

/// A `requires` naming something that is not in this plan is not an edge — it is already
/// on the machine, and inventing a node for it would install a package nobody froze.
#[test]
fn a_requirement_outside_the_plan_adds_nothing() {
    let installs = vec![spec("apt", "nginx", &["apt:already-there"])];
    let changes = saved_plan_to_changes(&installs, &[]);
    assert_eq!(changes.graph.node_count(), 1);
    assert_eq!(changes.graph.edge_count(), 0);
}
