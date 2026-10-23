use super::*;

#[test]
fn leaves_unknown_and_flag_first_tokens_alone() {
    let aliases = HashMap::new();
    let known = HashSet::new();
    assert_eq!(
        expand_command_aliases(argv(&["shall", "--version"]), &aliases, &known),
        argv(&["shall", "--version"])
    );
    assert_eq!(
        expand_command_aliases(argv(&["shall", "notanalias"]), &aliases, &known),
        argv(&["shall", "notanalias"])
    );
}
