use super::*;

#[test]
fn expands_alias_after_a_value_taking_global_flag() {
    let mut aliases = HashMap::new();
    aliases.insert("up".to_string(), "upgrade --all".to_string());
    let known: HashSet<String> = ["upgrade".to_string()].into_iter().collect();

    let out = expand_command_aliases(argv(&["shall", "-c", "/c.toml", "up"]), &aliases, &known);
    assert_eq!(out, argv(&["shall", "-c", "/c.toml", "upgrade", "--all"]));
}
