use super::*;

/// `plan --help` promises "the exact plan you inspect is the one you later apply", and an
/// ordering is part of a plan. This rebuilt the graph with `add_node` and no edges, so a
/// `@requires` held when `sync` applied it and was dropped when `apply` did — silently,
/// because an edgeless graph runs perfectly well in the wrong order.
#[test]
fn a_frozen_plan_keeps_the_ordering_its_specs_declare() {
    let installs = vec![
        spec("apt", "nginx", &["apt:libfoo"]),
        spec("apt", "libfoo", &[]),
    ];
    let changes = saved_plan_to_changes(&installs, &[]);

    assert_eq!(changes.graph.node_count(), 2);
    assert_eq!(
        changes.graph.edge_count(),
        1,
        "the `@requires` in the plan file was read back and dropped"
    );
    let libfoo = changes.install_map["apt:libfoo"];
    let nginx = changes.install_map["apt:nginx"];
    assert!(
        changes.graph.contains_edge(libfoo, nginx),
        "the edge must run requirement -> dependent, or the batch is ordered backwards"
    );
}
