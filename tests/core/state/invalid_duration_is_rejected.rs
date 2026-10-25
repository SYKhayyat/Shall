use super::*;

#[test]
fn invalid_duration_is_rejected() {
    let mut r = reg();
    assert!(r.suspend("apt", "htop", None, Some("banana")).is_err());
    assert!(r.list_suspensions().is_empty());
}
