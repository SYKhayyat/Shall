use super::*;

#[test]
fn options_are_independent() {
    assert_eq!(
        actions_for(None, Some("running")),
        vec![ServiceAction::Start]
    );
    assert_eq!(
        actions_for(None, Some("stopped")),
        vec![ServiceAction::Stop]
    );
    assert_eq!(actions_for(Some("true"), None), vec![ServiceAction::Enable]);
    assert_eq!(
        actions_for(Some("false"), None),
        vec![ServiceAction::Disable]
    );
    assert_eq!(
        actions_for(Some("true"), Some("running")),
        vec![ServiceAction::Enable, ServiceAction::Start]
    );
    assert_eq!(
        actions_for(None, Some("restarted")),
        vec![ServiceAction::Restart]
    );
}
