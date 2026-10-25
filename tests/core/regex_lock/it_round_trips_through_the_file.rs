use super::*;

#[test]
fn it_round_trips_through_the_file() {
    let tmp = TempDir::new().unwrap();
    let path = RegexLock::path_in(&tmp.path().join("locks"));
    assert!(RegexLock::load(&path).unwrap().is_empty());
    let mut lock = RegexLock::new();
    lock.record("apt", "^fonts-", vec!["fonts-a".into(), "fonts-b".into()]);
    lock.save(&path).unwrap();
    assert_eq!(
        RegexLock::load(&path).unwrap().get("apt", "^fonts-"),
        Some(&["fonts-a".to_string(), "fonts-b".to_string()][..])
    );
}
