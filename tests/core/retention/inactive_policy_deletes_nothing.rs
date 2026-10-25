use super::*;

#[test]
fn inactive_policy_deletes_nothing() {
    let p = RetentionPolicy::default();
    let items = vec![item("a", 100), item("b", 200), item("c", 300)];
    assert!(p.select_deletions(&items, now()).is_empty());
}
