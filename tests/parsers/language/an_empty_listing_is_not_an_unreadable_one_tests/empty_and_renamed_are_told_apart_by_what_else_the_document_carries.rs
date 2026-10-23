use super::*;

/// **The boundary between the two, side by side**, because the first attempt at the empty
/// fix erased it: it made every `dependencies`-less object read as zero, which would have
/// waved a renamed container through as an empty machine. Same shape of document, opposite
/// readings, and the thing that separates them is whether anything else in it could have
/// been the map.
#[test]
fn empty_and_renamed_are_told_apart_by_what_else_the_document_carries() {
    // Nothing else in it: empty.
    assert_eq!(
        parse_installed("npm", r#"{"name":"lib","version":"1.0.0"}"#).expect("empty"),
        Vec::new()
    );
    // A key holding a map of objects: that is the map under another name, and it is refused.
    assert!(parse_installed("npm", r#"{"name":"lib","packages":{"ts":{"version":"5"}}}"#).is_err());
    // A non-empty key that is NOT a map of objects cannot be the container, so it does not
    // trip the guard — `problems` is an array npm really does emit.
    assert_eq!(
        parse_installed("npm", r#"{"name":"lib","problems":["something"]}"#).expect("empty"),
        Vec::new()
    );
}
