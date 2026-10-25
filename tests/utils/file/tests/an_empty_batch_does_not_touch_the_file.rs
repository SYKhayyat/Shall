use super::*;

/// An empty batch writes nothing and says so, rather than opening the file to add a
/// newline. A wave with no members is an ordinary thing, and it must not touch the log.
#[test]
fn an_empty_batch_does_not_touch_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("untouched.jsonl");
    assert!(!append_lines(&path, &[]).unwrap());
    assert!(
        !path.exists(),
        "an empty batch created the file; a wave with nothing in it must leave no trace"
    );
}
