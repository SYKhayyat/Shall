use super::*;

#[tokio::test]
async fn an_install_and_a_removal_never_share_a_command() {
    let mut graph = StableDiGraph::new();
    graph.add_node(GraphAction::Install(spec("apt", "jq")));
    graph.add_node(GraphAction::Remove {
        name: "nano".into(),
        backend: "apt".into(),
    });
    let mut h = harness(graph, &["apt"]).await;
    h.tx.execute_with_telemetry().await.unwrap();

    assert_eq!(
        h.counters[0].calls.load(Ordering::SeqCst),
        2,
        "installing and removing are two different commands to the same manager"
    );
    assert_eq!(h.counters[0].widest.load(Ordering::SeqCst), 1);
}
