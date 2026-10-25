use super::*;

#[test]
fn a_name_nobody_declares_any_more_is_forgotten() {
    let mut lock = BareLock::new();
    lock.record("ripgrep", "cargo");
    lock.record("gone", "apt");
    assert!(lock.retain_declared(&["ripgrep".to_string()]));
    assert_eq!(lock.get("gone"), None);
    assert!(!lock.retain_declared(&["ripgrep".to_string()]), "no change");
}
