use super::*;

/// The claim is *whether*, so a second install of the same package is not a second line.
/// A file that grew per attempt would answer "what did Shall put here" with a count of tries.
#[test]
fn installing_the_same_package_twice_is_one_receipt() {
    let tmp = tempdir().unwrap();
    let path = tmp.path().join(Receipts::FILE_NAME);
    let mut receipts = Receipts::at(path.clone()).unwrap();
    assert!(
        receipts.record("apt", "jq", 100).unwrap(),
        "the first is new"
    );
    assert!(
        !receipts.record("apt", "jq", 200).unwrap(),
        "the second says nothing new"
    );

    let body = std::fs::read_to_string(&path).unwrap();
    assert_eq!(
        body.lines().filter(|l| !l.trim().is_empty()).count(),
        1,
        "one line on disk, not one per attempt:\n{body}"
    );
    // And the timestamp kept is the first — when it arrived, not when it was re-installed.
    assert_eq!(
        Receipts::at(path)
            .unwrap()
            .entries()
            .find(|r| r.key() == "apt:jq")
            .unwrap()
            .at_unix,
        100
    );
}
