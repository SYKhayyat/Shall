use super::*;

#[test]
fn the_receipts_file_is_the_wals_sibling() {
    assert_eq!(
        Receipts::sibling_of(Path::new("/data/journal.jsonl")),
        PathBuf::from("/data/receipts.jsonl")
    );
    // A journal path with no directory beside it still lands somewhere writable-relative
    // rather than panicking on `parent()` returning None.
    assert_eq!(
        Receipts::sibling_of(Path::new("journal.jsonl")),
        PathBuf::from("receipts.jsonl")
    );
}
