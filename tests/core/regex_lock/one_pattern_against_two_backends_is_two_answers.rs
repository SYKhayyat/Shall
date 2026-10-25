use super::*;

#[test]
fn one_pattern_against_two_backends_is_two_answers() {
    let mut lock = RegexLock::new();
    lock.record("apt", "^fonts-", vec!["fonts-a".into()]);
    lock.record("pacman", "^fonts-", vec!["fonts-z".into()]);
    assert_eq!(
        lock.get("apt", "^fonts-"),
        Some(&["fonts-a".to_string()][..])
    );
    assert_eq!(
        lock.get("pacman", "^fonts-"),
        Some(&["fonts-z".to_string()][..])
    );
}
