use super::*;

#[test]
fn a_receipt_round_trips_through_the_file() {
    let tmp = tempdir().unwrap();
    let path = tmp.path().join(Receipts::FILE_NAME);
    let mut written = Receipts::at(path.clone()).unwrap();
    assert!(written.record("cargo", "hexyl", 1).unwrap());

    let read = Receipts::at(path).unwrap();
    assert!(read.claims("cargo", "hexyl"));
    assert!(
        !read.claims("cargo", "fd"),
        "a receipt is about one package"
    );
    assert_eq!(read.len(), 1);
    assert_eq!(read.keys().collect::<Vec<_>>(), ["cargo:hexyl"]);
}
