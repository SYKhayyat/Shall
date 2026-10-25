use super::*;

#[test]
fn a_pattern_nobody_declares_any_more_is_forgotten() {
    let mut lock = RegexLock::new();
    lock.record("apt", "^fonts-", vec!["fonts-a".into()]);
    lock.record("apt", "^gone-", vec!["gone-a".into()]);
    assert!(lock.retain_declared(&[key("apt", "^fonts-")]));
    assert_eq!(lock.get("apt", "^gone-"), None);
}
