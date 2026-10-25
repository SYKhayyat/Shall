use super::*;

#[test]
fn it_round_trips_through_the_file() {
    let tmp = TempDir::new().unwrap();
    let path = BareLock::path_in(&tmp.path().join("locks"));
    let mut lock = BareLock::new();
    lock.record("ripgrep", "cargo");
    lock.save(&path).unwrap();
    assert_eq!(BareLock::load(&path).unwrap().get("ripgrep"), Some("cargo"));
}
