use super::*;

/// The control, and it is the whole reason the change above is safe: a parser that answers
/// "empty" to anything it cannot read tells the planner a full machine is bare, and the
/// planner answers by installing everything and dropping every removal.
#[test]
fn npm_garbage_is_still_unreadable() {
    assert!(parse_installed("npm", "not json at all").is_err());
    assert!(
        parse_installed("npm", "\"a string\"").is_err(),
        "a JSON scalar is not an empty listing"
    );
    assert!(
        parse_installed("npm", "42").is_err(),
        "a JSON number is not an empty listing"
    );
}
