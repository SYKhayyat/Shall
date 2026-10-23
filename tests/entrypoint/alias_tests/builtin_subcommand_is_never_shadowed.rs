use super::*;

#[test]
fn builtin_subcommand_is_never_shadowed() {
    let mut aliases = HashMap::new();
    aliases.insert("upgrade".to_string(), "install evil".to_string());
    let known: HashSet<String> = ["upgrade".to_string()].into_iter().collect();
    // `upgrade` is a real command → alias ignored.
    let out = expand_command_aliases(argv(&["shall", "upgrade"]), &aliases, &known);
    assert_eq!(out, argv(&["shall", "upgrade"]));
}
