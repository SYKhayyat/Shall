use super::*;

/// `-S` is the pickaxe: commits where the number of occurrences CHANGED. `git log <path>`
/// would list every commit that touched the file, which for a busy module is all of them.
#[test]
fn the_query_uses_the_pickaxe_and_is_limited_to_the_given_paths() {
    let cmd = argv("ripgrep", &["modules", "profiles"]);
    assert!(cmd.iter().any(|a| a == "-S"), "{:?}", cmd);
    assert!(cmd.iter().any(|a| a == "ripgrep"), "{:?}", cmd);
    let sep = cmd
        .iter()
        .position(|a| a == "--")
        .expect("a path separator");
    assert_eq!(
        &cmd[sep + 1..],
        &["modules".to_string(), "profiles".to_string()]
    );
}
