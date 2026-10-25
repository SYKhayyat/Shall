use super::*;

/// U2 + `Q44`: **a definition works with the batch verb and without it.**
///
/// With `outdated_args`, the manager is asked once. Without, the caller falls back to
/// asking per package — slower, but an answer. What must never happen is the third case:
/// a definition that declared an outdated verb and got no `Searchable` at all, because the
/// capability was gated on `search_args`. Its updates were then silently never reported,
/// which looks exactly like a machine with nothing out of date.
#[tokio::test]
async fn a_definition_reports_updates_with_a_batch_verb_and_without_one() {
    let toml_src = r#"
[[backend]]
name = "corp"
binary = "corpctl"
install_args = ["add"]
list_args = ["ls"]
outdated_args = ["ls", "--stale"]
[backend.parser]
format = "columns"
name_col = 0
version_col = 1
"#;
    let parsed: CustomBackendsFile =
        toml::from_str(toml_src).expect("a definition using the new keys must parse");
    let def = parsed.backend.into_iter().next().unwrap();

    let vfs = std::sync::Arc::new(dashmap::DashMap::new());
    let mock = std::sync::Arc::new(shall::core::executor::MockExecutor::new(vfs.clone()));
    mock.set_response(
        "corpctl ls --stale",
        Ok(shall::core::executor::DryRunOutput {
            stdout: b"widget 2.0
gadget 3.1
"
            .to_vec(),
            stderr: vec![],
        }
        .into()),
    );
    let exec = CommandExecutor::with_layer(
        false,
        false,
        mock.clone(),
        vfs,
        std::sync::Arc::new(dashmap::DashMap::new()),
    );
    let parser = parser_for(&def).unwrap();
    let caps = build_capabilities_with(def, &exec, parser);

    // It declared no `search_args`, and it is searchable anyway — because the outdated
    // verb lives on that capability.
    let s = caps
        .as_searchable()
        .expect("a definition with an outdated verb must be reachable through Searchable");
    let stale = s
        .outdated_all()
        .await
        .expect("the probe runs")
        .expect("a declared verb means Some, never None");
    let names: Vec<&str> = stale.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, vec!["widget", "gadget"]);
    assert_eq!(stale[0].version.as_deref(), Some("2.0"));

    // ...and `search` itself is refused by name rather than answered as "no results",
    // because it was never configured.
    let err = s.search("widget").await.unwrap_err().to_string();
    assert!(err.contains("not told how to search"), "{err}");
}
