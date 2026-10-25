use super::*;

#[test]
fn openrc_uses_rc_update_and_rc_service() {
    let p = shipped("openrc");
    assert_eq!(
        p.plan(ServiceAction::Enable, "sshd"),
        vec![(
            ServiceAction::Enable,
            vec![
                "rc-update".to_string(),
                "add".into(),
                "sshd".into(),
                "default".into()
            ]
        )]
    );
    assert_eq!(
        p.plan(ServiceAction::Start, "sshd"),
        vec![(
            ServiceAction::Start,
            vec!["rc-service".to_string(), "sshd".into(), "start".into()]
        )]
    );
}
