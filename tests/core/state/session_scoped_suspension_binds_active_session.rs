use super::*;

#[test]
fn session_scoped_suspension_binds_active_session() {
    let mut r = reg();
    r.active_session_id = Some("shell-abc".into());
    let at = r.suspend("dnf", "nano", None, None).unwrap();
    assert!(
        at.is_none(),
        "session-scoped suspension has no timed restore"
    );
    let s = &r.list_suspensions()[0];
    assert_eq!(s.session_id.as_deref(), Some("shell-abc"));
    assert_eq!(r.get_session_suspensions("shell-abc").len(), 1);
    assert_eq!(r.get_session_suspensions("other").len(), 0);
}
