use super::*;

/// **`U41`, both halves, as one question.** Rollback does not undo work that moved the
/// machine toward the declared state â€” an install that succeeded of something still
/// declared, or a removal that succeeded of something still undeclared. The install arm had
/// this rule and the removal arm did not, and nothing in the register said the pair had
/// come apart.
#[tokio::test]
async fn one_rollback_rule_answers_both_directions() {
    let mut graph = StableDiGraph::new();
    graph.add_node(GraphAction::Install(spec("apt", "jq")));
    let h = harness(graph, &["apt"]).await;

    // Nothing to reconcile against: no answer, and both arms fall back to compensating.
    assert_eq!(h.tx.plan_intends_present("apt", "jq"), None);
    assert_eq!(h.tx.plan_intends_present("apt", "vim"), None);

    let declared: std::collections::HashSet<String> = ["apt:jq".to_string()].into_iter().collect();
    let tx = h.tx.reconciling(Arc::new(declared));

    // The install arm: `jq` installed cleanly and is still declared, so the removal that
    // would compensate it is skipped.
    assert_eq!(
        tx.plan_intends_present("apt", "jq"),
        Some(true),
        "an install of something still declared is the goal reached early"
    );
    // The removal arm: nothing declares `vim`, which is why it was removed, and that fact
    // has not changed â€” so the reinstate that would compensate it is skipped.
    assert_eq!(
        tx.plan_intends_present("apt", "vim"),
        Some(false),
        "a removal of something still undeclared must not be put back"
    );
    // Keyed by backend and name together, so `apt:jq` does not answer for `cargo:jq`.
    assert_eq!(tx.plan_intends_present("cargo", "jq"), Some(false));
}
