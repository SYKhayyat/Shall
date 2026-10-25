use super::*;

/// An absent file is an empty record, which is the ordinary state of a machine where Shall
/// has installed nothing — not an error, and not a reason to fail a command.
#[test]
fn no_file_is_an_empty_record() {
    let tmp = tempdir().unwrap();
    let receipts = Receipts::at(tmp.path().join("never-written.jsonl")).unwrap();
    assert!(receipts.is_empty());
    assert!(!receipts.claims("apt", "jq"));
}
