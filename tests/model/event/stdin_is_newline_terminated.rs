use super::*;

/// A hook that reads a line at a time must see a complete record.
#[test]
fn stdin_is_newline_terminated() {
    let out = Payload::new(Event::AfterSync, json!({})).to_stdin();
    assert!(out.ends_with('\n'), "{:?}", out);
    assert_eq!(out.lines().count(), 1, "{:?}", out);
}
