use super::*;

#[test]
fn timed_suspension_sets_future_restore_and_no_session() {
    let mut r = reg();
    let at = r
        .suspend("apt", "htop", Some("3.0".into()), Some("1h"))
        .unwrap();
    assert!(at.is_some());
    assert!(at.unwrap() > StateRegistry::now());
    let s = &r.list_suspensions()[0];
    assert_eq!(s.backend, "apt");
    assert_eq!(s.name, "htop");
    assert_eq!(s.version.as_deref(), Some("3.0"));
    assert!(s.session_id.is_none());
}
