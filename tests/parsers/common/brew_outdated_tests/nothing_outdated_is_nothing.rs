use super::*;

#[test]
fn nothing_outdated_is_nothing() {
    assert!(parse_brew_outdated("").is_empty());
    assert!(parse_brew_outdated("==> Auto-updating Homebrew...\n").is_empty());
    assert!(parse_brew_outdated(r#"{"formulae":[],"casks":[]}"#).is_empty());
}
