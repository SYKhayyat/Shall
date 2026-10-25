use super::*;

#[test]
fn it_round_trips_through_the_file_and_creates_the_directory() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("locks").join("probe.toml");
    let mut p = Probe::new();
    p.entries.insert("a".to_string(), "b".into());
    p.save(&path).unwrap();
    assert!(path.exists(), "save did not create locks/");
    assert_eq!(Probe::load(&path).unwrap(), p);
}
