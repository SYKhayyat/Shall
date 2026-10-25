use super::*;

#[test]
fn a_requires_cycle_names_the_packages_and_where_they_came_from() {
    // V.45: the message must name what closed the loop, not just say one exists.
    let mut graph: StableDiGraph<GraphAction, ()> = StableDiGraph::new();
    let mk = |name: &str, src: &str| {
        let mut options = shall::config::grammar::Options::default();
        options.set("__source", src.to_string());
        GraphAction::Install(PackageSpec {
            name: name.into(),
            backend: "apt".into(),
            options,
            requires: vec![],
            present: true,
        })
    };
    let a = graph.add_node(mk("foo", "modules/dev.txt:3"));
    let b = graph.add_node(mk("bar", "modules/dev.txt:4"));
    graph.add_edge(a, b, ());
    graph.add_edge(b, a, ());

    // II.7: a `requires` loop owes the same error a `use` loop does — every file and
    // line, in the order the edges point, and the arrow back to where it started.
    let msg = describe_cycle(&graph);
    assert!(
        msg.contains("modules/dev.txt:3  apt:foo requires apt:bar"),
        "{}",
        msg
    );
    assert!(
        msg.contains("modules/dev.txt:4  apt:bar requires apt:foo"),
        "{}",
        msg
    );
    assert!(msg.trim_end().ends_with("^ back to apt:foo"), "{}", msg);
}
