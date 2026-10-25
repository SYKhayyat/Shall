use super::*;

/// A file whose last line was written without a newline gets one before the batch, not
/// between every pair of lines. The single-line version already did this; the batch has to
/// do it exactly once or it writes a blank line into the log on every call.
#[test]
fn a_batch_repairs_a_torn_tail_once_and_not_per_line() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("torn.jsonl");
    std::fs::write(&path, "half-written").unwrap();

    assert!(append_lines(&path, &["a", "b"]).unwrap());
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "half-written\na\nb\n"
    );
}
