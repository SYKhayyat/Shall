use super::*;

#[test]
fn a_missing_file_is_an_empty_lock_and_not_an_error() {
    let tmp = TempDir::new().unwrap();
    let path = BareLock::path_in(&tmp.path().join("locks"));
    assert!(BareLock::load(&path).unwrap().is_empty());
}
