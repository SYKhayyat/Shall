use super::*;

#[test]
fn the_per_user_domain_is_expressed_the_way_each_init_needs() {
    let actions = [
        ServiceAction::Enable,
        ServiceAction::Disable,
        ServiceAction::Start,
        ServiceAction::Stop,
        ServiceAction::Restart,
    ];
    let sd = shipped("systemd");
    for action in actions {
        let user = sd.plan_in(action, "thing", Domain::User);
        assert!(!user.is_empty(), "systemd/{action:?}: no per-user argv");
        assert_ne!(
            sd.plan(action, "thing"),
            user,
            "systemd/{action:?}: the per-user action is the system one, so it would act on \
             the machine rather than on the account"
        );
        for (_, cmd) in &user {
            assert!(
                cmd.iter().any(|a| a == "--user"),
                "systemd/{action:?} does not reach a per-user bus: {cmd:?}"
            );
        }
    }
    let ld = shipped("launchd");
    for action in actions {
        let user = ld.plan_in(action, "thing", Domain::User);
        assert!(
            !user.is_empty(),
            "launchd/{action:?}: no per-user argv, so a `@user=` line is refused by name"
        );
    }
}
