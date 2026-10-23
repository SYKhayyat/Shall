use super::*;

/// The control: a plan whose specs require nothing goes on one command line. This is the
/// same property from the other side — an edge that should not exist costs a whole extra
/// manager invocation, measured at ten times the batched cost (V.115).
#[test]
fn a_plan_with_no_requires_has_no_edges_to_split_the_batch() {
    let installs = vec![spec("apt", "htop", &[]), spec("apt", "jq", &[])];
    let changes = saved_plan_to_changes(&installs, &[]);
    assert_eq!(changes.graph.edge_count(), 0);
}
