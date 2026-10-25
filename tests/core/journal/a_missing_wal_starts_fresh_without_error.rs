use super::*;

#[test]
fn a_missing_wal_starts_fresh_without_error() {
    let tmp = tempdir().unwrap();
    let journal = Journal::at(tmp.path().join("nope.json")).unwrap();
    assert!(!journal.needs_recovery());
}
