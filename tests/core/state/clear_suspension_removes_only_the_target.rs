use super::*;

#[test]
fn clear_suspension_removes_only_the_target() {
    let mut r = reg();
    r.suspend("apt", "a", None, Some("1h")).unwrap();
    r.suspend("apt", "b", None, Some("1h")).unwrap();
    assert!(r.clear_suspension("apt", "a"));
    assert!(!r.clear_suspension("apt", "a"), "already gone");
    assert_eq!(r.list_suspensions().len(), 1);
    assert_eq!(r.list_suspensions()[0].name, "b");
}
