use super::*;

/// The engine's own shape, measured on plans whose right answer is arithmetic.
///
/// This is the `Mutating` half of the latency gate, and it is here rather than in
/// `latency.rs` because the two numbers it compares are the engine's: one is the graph's
/// longest chain, the other is how many passes the loop actually had work to hand out.
/// `latency.rs` owns the rule; nothing owned the measurement, which is why the class read as
/// exempt for as long as it did.
///
/// A wide plan must take one wave however many packages are in it â€” that is the assertion a
/// change awaiting the batch in flight before dispatching more would break, while leaving
/// every package installed and every other test green. Watched failing against exactly that
/// change: six packages, depth one, six waves.
#[tokio::test]
async fn the_engine_reports_the_shape_it_actually_ran_in() {
    // Wide: nothing depends on anything, so one round is the only correct answer.
    let mut wide = StableDiGraph::new();
    for i in 0..6 {
        wide.add_node(GraphAction::Install(spec("apt", &format!("pkg{}", i))));
    }
    let mut h = harness(wide, &["apt"]).await;
    h.tx.execute_with_telemetry().await.unwrap();
    assert_eq!(
        h.tx.last_scheduling,
        Some(Scheduling {
            packages: 6,
            depth: 1,
            waves: 1
        }),
        "six independent packages took more than one wave to hand out"
    );

    // A chain of three. Serial is the correct schedule here, and the rule must not report
    // it â€” which is the case a threshold read off a host gets wrong.
    let mut chain = StableDiGraph::new();
    let a = chain.add_node(GraphAction::Install(spec("apt", "libfoo")));
    let b = chain.add_node(GraphAction::Install(spec("apt", "foo-tool")));
    let c = chain.add_node(GraphAction::Install(spec("apt", "foo-plugin")));
    chain.add_edge(a, b, ());
    chain.add_edge(b, c, ());
    let mut h = harness(chain, &["apt"]).await;
    h.tx.execute_with_telemetry().await.unwrap();
    let ran =
        h.tx.last_scheduling
            .expect("a completed run records its shape");
    assert_eq!(ran.depth, 3, "a chain of three has three levels");
    assert_eq!(ran.waves, 3, "and goes idle once per level");
    assert!(
        shall::core::latency::scheduling_violation(ran.packages, ran.depth, ran.waves).is_none(),
        "a chain executed one at a time is the plan's shape, not a regression"
    );

    // A diamond: one, then two together, then one. Four packages, three levels.
    let mut diamond = StableDiGraph::new();
    let root = diamond.add_node(GraphAction::Install(spec("apt", "base")));
    let left = diamond.add_node(GraphAction::Install(spec("apt", "left")));
    let right = diamond.add_node(GraphAction::Install(spec("apt", "right")));
    let top = diamond.add_node(GraphAction::Install(spec("apt", "top")));
    diamond.add_edge(root, left, ());
    diamond.add_edge(root, right, ());
    diamond.add_edge(left, top, ());
    diamond.add_edge(right, top, ());
    let mut h = harness(diamond, &["apt"]).await;
    h.tx.execute_with_telemetry().await.unwrap();
    let ran =
        h.tx.last_scheduling
            .expect("a completed run records its shape");
    assert_eq!(
        (ran.packages, ran.depth),
        (4, 3),
        "four packages across three levels"
    );
    assert!(
        shall::core::latency::scheduling_violation(ran.packages, ran.depth, ran.waves).is_none(),
        "a diamond run in {} waves against a depth of 3",
        ran.waves
    );

    // Two independent chains. **This is the case that killed the first version of the
    // rule**, which counted dispatches: `a` and `b` start together, whichever finishes
    // first hands out its child while the other pair is still running, and the pass count
    // reaches three against a depth of two â€” a violation reported against a scheduler doing
    // precisely what it should. Counting idle restarts instead, the engine never runs dry
    // here at all.
    let mut chains = StableDiGraph::new();
    let a = chains.add_node(GraphAction::Install(spec("apt", "a")));
    let b = chains.add_node(GraphAction::Install(spec("apt", "b")));
    let a2 = chains.add_node(GraphAction::Install(spec("apt", "a2")));
    let b2 = chains.add_node(GraphAction::Install(spec("apt", "b2")));
    chains.add_edge(a, a2, ());
    chains.add_edge(b, b2, ());
    let mut h = harness(chains, &["apt"]).await;
    h.tx.execute_with_telemetry().await.unwrap();
    let ran =
        h.tx.last_scheduling
            .expect("a completed run records its shape");
    assert_eq!((ran.packages, ran.depth), (4, 2));
    assert!(
        ran.waves <= ran.depth,
        "two independent chains reported {} wave(s) against a depth of {}",
        ran.waves,
        ran.depth
    );
}
