use super::*;

#[test]
fn options_default_to_enable_and_start() {
    assert_eq!(
        actions_for(None, None),
        vec![ServiceAction::Enable, ServiceAction::Start]
    );
}
