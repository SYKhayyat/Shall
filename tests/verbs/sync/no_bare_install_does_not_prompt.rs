use super::*;

#[test]
fn no_bare_install_does_not_prompt() {
    let changes = shall::app::sync::planner::SyncChanges::default();
    let mut asked = false;
    // `never` too: a setting about bare declarations has nothing to say about a plan that has
    // none, and reporting `Declined` there would make an ordinary sync look like a refusal.
    for policy in [
        shall::config::config::BareInstall::Ask,
        shall::config::config::BareInstall::Never,
    ] {
        let decision = confirm_bare_installs_with(&changes, policy, false, |_| {
            asked = true;
            Ok(false)
        })
        .unwrap();
        assert_eq!(decision, BareDecision::Proceed);
    }
    assert!(!asked);
}
