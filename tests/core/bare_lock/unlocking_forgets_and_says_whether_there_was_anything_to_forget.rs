use super::*;

#[test]
fn unlocking_forgets_and_says_whether_there_was_anything_to_forget() {
    let mut lock = BareLock::new();
    lock.record("ripgrep", "cargo");
    assert!(lock.forget("ripgrep"));
    assert_eq!(lock.get("ripgrep"), None);
    assert!(!lock.forget("ripgrep"), "forgetting twice is not a change");

    lock.record("jq", "apt");
    assert!(lock.clear());
    assert!(lock.is_empty());
    assert!(!lock.clear());
}
