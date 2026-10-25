use super::*;

#[test]
fn a_missing_file_loads_empty_rather_than_failing() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("locks").join("nothing-here.toml");
    assert_eq!(Probe::load(&path).unwrap(), Probe::new());
}
