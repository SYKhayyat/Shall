use super::*;

/// **The property the file exists for, stated as a test rather than a comment: a receipt
/// outlives the journal purge.** Written with a timestamp far enough in the past that the
/// seven-day rule would have dropped its counterpart, and still read back.
#[test]
fn a_receipt_is_still_there_a_year_later() {
    let tmp = tempdir().unwrap();
    let path = tmp.path().join(Receipts::FILE_NAME);
    let old = 1_600_000_000; // 2020-09-13, well past any age rule.
    let mut written = Receipts::at(path.clone()).unwrap();
    written.record("apt", "jq", old).unwrap();

    let read = Receipts::at(path).unwrap();
    assert!(read.claims("apt", "jq"));
    assert_eq!(
        read.entries()
            .find(|r| r.key() == "apt:jq")
            .unwrap()
            .at_unix,
        old
    );
}
