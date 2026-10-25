use super::*;

#[test]
fn a_recorded_name_keeps_its_backend() {
    let mut lock = BareLock::new();
    assert!(lock.record("ripgrep", "cargo"));
    assert_eq!(lock.get("ripgrep"), Some("cargo"));
    // Recording the same answer is not a change: an unchanged lock must not be rewritten,
    // or every sync becomes a commit.
    assert!(!lock.record("ripgrep", "cargo"));
    assert!(lock.record("ripgrep", "apt"));
}
