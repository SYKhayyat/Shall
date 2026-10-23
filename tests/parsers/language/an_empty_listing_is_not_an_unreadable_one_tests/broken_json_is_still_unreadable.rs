use super::*;

/// A document that does NOT parse is still unread, however many braces it opens with.
#[test]
fn broken_json_is_still_unreadable() {
    assert!(parse_installed("pipx", "{\n  \"venvs\": {\n").is_err());
}
