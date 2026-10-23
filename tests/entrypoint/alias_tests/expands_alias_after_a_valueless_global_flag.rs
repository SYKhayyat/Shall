use super::*;

#[test]
fn expands_alias_after_a_valueless_global_flag() {
    // `--progress` is a bool: clap gives it SetTrue, so it consumes no value. The old
    // hand-written flag list claimed it took one, so `i += 2` walked past `up` and the
    // alias silently never expanded.
    let mut aliases = HashMap::new();
    aliases.insert("up".to_string(), "upgrade --all".to_string());
    let known: HashSet<String> = ["upgrade".to_string()].into_iter().collect();

    let out = expand_command_aliases(argv(&["shall", "--progress", "up"]), &aliases, &known);
    assert_eq!(out, argv(&["shall", "--progress", "upgrade", "--all"]));
}
