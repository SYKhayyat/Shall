use super::*;

#[test]
fn systemd_maps_each_action_and_ends_its_options_before_the_unit() {
    let sd = shipped("systemd");
    for (action, verb) in [
        (ServiceAction::Enable, "enable"),
        (ServiceAction::Disable, "disable"),
        (ServiceAction::Start, "start"),
        (ServiceAction::Stop, "stop"),
        (ServiceAction::Restart, "restart"),
    ] {
        assert_eq!(
            sd.plan(action, "nginx"),
            vec![(
                action,
                vec![
                    "systemctl".to_string(),
                    "--no-pager".into(),
                    verb.into(),
                    "--".into(),
                    "nginx".into()
                ]
            )]
        );
    }
}
