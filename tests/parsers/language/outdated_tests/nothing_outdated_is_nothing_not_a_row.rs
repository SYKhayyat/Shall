use super::*;

/// Nothing outdated is a real answer and every one of these has its own way of saying it.
#[test]
fn nothing_outdated_is_nothing_not_a_row() {
    assert!(parse_pip_outdated("[]").is_empty());
    assert!(parse_pip_outdated("").is_empty());
    assert!(parse_npm_outdated("{}", "npm").is_empty());
    assert!(parse_npm_outdated("", "npm").is_empty());
    assert!(parse_gem_outdated("").is_empty());
    // gem prints nothing at all when everything is current.
    assert!(parse_gem_outdated("\n\n").is_empty());
}
