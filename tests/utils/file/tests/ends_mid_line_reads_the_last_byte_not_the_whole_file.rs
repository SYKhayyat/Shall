use super::*;

/// The seek-based read answers like the whole-file read it replaced: an empty file and a
/// missing file both count as ending cleanly, and one long file still gets the newline
/// verdict from its final byte only. (The old form read every byte of that file to ask.)
#[test]
fn ends_mid_line_reads_the_last_byte_not_the_whole_file() {
    let dir = TempDir::new().unwrap();
    assert!(!ends_mid_line(&dir.path().join("absent")));
    let empty = dir.path().join("empty");
    std::fs::write(&empty, b"").unwrap();
    assert!(!ends_mid_line(&empty));
    let clean = dir.path().join("clean");
    std::fs::write(&clean, b"one\ntwo\n").unwrap();
    assert!(!ends_mid_line(&clean));
    let dirty = dir.path().join("dirty");
    std::fs::write(&dirty, b"one\ntwo").unwrap();
    assert!(ends_mid_line(&dirty));
}
