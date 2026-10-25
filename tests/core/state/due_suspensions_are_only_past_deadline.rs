use super::*;

#[test]
fn due_suspensions_are_only_past_deadline() {
    let mut r = reg();
    r.suspensions.push(Suspension {
        backend: "apt".into(),
        name: "due".into(),
        version: None,
        restore_at: Some(StateRegistry::now().saturating_sub(10)),
        session_id: None,
        suspended_at: StateRegistry::now(),
    });
    r.suspend("apt", "later", None, Some("30d")).unwrap();
    let due = r.get_due_suspensions();
    assert_eq!(due.len(), 1);
    assert_eq!(due[0].name, "due");
}
