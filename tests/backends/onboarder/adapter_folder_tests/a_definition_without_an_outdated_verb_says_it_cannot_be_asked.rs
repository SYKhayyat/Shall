use super::*;

/// Without the verb: `None`, which is the caller's signal to ask per package. It must not
/// be `Some(vec![])` — that would say "asked, nothing stale" and mark the whole backend
/// current.
#[tokio::test]
async fn a_definition_without_an_outdated_verb_says_it_cannot_be_asked() {
    let toml_src = r#"
[[backend]]
name = "corp3"
binary = "corpctl"
list_args = ["ls"]
search_args = ["find"]
[backend.parser]
format = "columns"
name_col = 0
version_col = 1
"#;
    let parsed: CustomBackendsFile = toml::from_str(toml_src).unwrap();
    let def = parsed.backend.into_iter().next().unwrap();
    let vfs = std::sync::Arc::new(dashmap::DashMap::new());
    let mock = std::sync::Arc::new(shall::core::executor::MockExecutor::new(vfs.clone()));
    let exec = CommandExecutor::with_layer(
        false,
        false,
        mock,
        vfs,
        std::sync::Arc::new(dashmap::DashMap::new()),
    );
    let parser = parser_for(&def).unwrap();
    let caps = build_capabilities_with(def, &exec, parser);
    let s = caps.as_searchable().expect("it declared search_args");
    assert!(
        s.outdated_all().await.unwrap().is_none(),
        "no verb means `cannot be asked`, never `asked and nothing is stale`"
    );
}
