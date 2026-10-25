use super::*;

/// `asdf plugin list` exits 0 and prints `No plugins installed`, so the exit code says
/// "yes" for every plugin that is missing. And a substring match would let `jqx` answer
/// for `jq`.
#[test]
fn an_output_probe_is_line_exact() {
    assert!(PrereqDef::output_satisfies("nodejs\njq\n", "jq"));
    assert!(!PrereqDef::output_satisfies("No plugins installed\n", "jq"));
    assert!(!PrereqDef::output_satisfies("jqx\n", "jq"));
}
