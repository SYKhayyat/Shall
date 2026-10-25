use super::*;

/// **`append_lines` writes every line and flushes once.**
///
/// B3: the WAL flushed per entry, so opening a wave of *k* packages cost *k* physical
/// flushes, serialised, under the journal mutex, before a single manager was invoked — ~298
/// of them on the 298-package config the planner's own comment cites. The guarantee that
/// matters is unchanged and is what this asserts: every line is on disk when the call
/// returns, in order, so recovery's promise that the record precedes the work still holds.
#[test]
fn several_lines_are_appended_in_order_and_all_of_them_arrive() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("wal.jsonl");

    assert!(append_lines(&path, &["one", "two", "three"]).unwrap());
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "one\ntwo\nthree\n",
        "a batch must be indistinguishable from the same lines appended one at a time"
    );

    // And a second batch continues rather than replacing — this is an append log, and the
    // failure mode of getting that wrong is losing every entry before the last wave.
    assert!(append_lines(&path, &["four"]).unwrap());
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "one\ntwo\nthree\nfour\n"
    );
}
