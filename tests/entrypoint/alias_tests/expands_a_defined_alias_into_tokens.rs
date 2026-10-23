use super::*;

#[test]
fn expands_a_defined_alias_into_tokens() {
    let mut aliases = HashMap::new();
    aliases.insert("up".to_string(), "upgrade --all".to_string());
    let known: HashSet<String> = ["upgrade".to_string()].into_iter().collect();

    let out = expand_command_aliases(argv(&["shall", "up", "--dry-run"]), &aliases, &known);
    assert_eq!(out, argv(&["shall", "upgrade", "--all", "--dry-run"]));
}
