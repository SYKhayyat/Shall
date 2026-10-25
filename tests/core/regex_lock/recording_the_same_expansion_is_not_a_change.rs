use super::*;

#[test]
fn recording_the_same_expansion_is_not_a_change() {
    let mut lock = RegexLock::new();
    assert!(lock.record("apt", "^fonts-", vec!["b".into(), "a".into()]));
    // Sorted on the way in, so the order the manager happened to print in is not a diff.
    assert!(!lock.record("apt", "^fonts-", vec!["a".into(), "b".into()]));
    assert!(lock.record("apt", "^fonts-", vec!["a".into()]));
}
