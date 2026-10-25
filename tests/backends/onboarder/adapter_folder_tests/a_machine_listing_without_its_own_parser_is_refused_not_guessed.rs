use super::*;

/// A machine-readable listing without a parser for it is refused by name rather than read
/// with the *text* parser — which would hand JSON to a column reader, find nothing, and
/// report an empty machine (`Q40`'s class, arriving through a config file).
#[test]
fn a_machine_listing_without_its_own_parser_is_refused_not_guessed() {
    let toml_src = r#"
[[backend]]
name = "corp2"
list_args = ["ls"]
machine_list_args = ["ls", "--json"]
[backend.parser]
format = "columns"
name_col = 0
version_col = 1
"#;
    let parsed: CustomBackendsFile = toml::from_str(toml_src).unwrap();
    let def = parsed.backend.into_iter().next().unwrap();
    assert!(def.machine_list_args.is_some());
    assert!(
        def.machine_list_parser.is_none(),
        "the fixture is the case being guarded: args without a parser"
    );

    let vfs = std::sync::Arc::new(dashmap::DashMap::new());
    let mock = std::sync::Arc::new(shall::core::executor::MockExecutor::new(vfs.clone()));
    let exec = CommandExecutor::with_layer(
        false,
        false,
        mock,
        vfs,
        std::sync::Arc::new(dashmap::DashMap::new()),
    );
    // Builds without panicking, and falls back to `list_args`; the warning names the file.
    let parser = parser_for(&def).unwrap();
    let _ = build_capabilities_with(def, &exec, parser);
}
