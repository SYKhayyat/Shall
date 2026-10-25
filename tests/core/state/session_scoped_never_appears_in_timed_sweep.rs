use super::*;

#[test]
fn session_scoped_never_appears_in_timed_sweep() {
    let mut r = reg();
    r.active_session_id = Some("s1".into());
    r.suspend("apt", "nano", None, None).unwrap();
    assert!(r.get_due_suspensions().is_empty());
}
