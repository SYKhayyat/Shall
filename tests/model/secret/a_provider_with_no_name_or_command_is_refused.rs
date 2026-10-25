use super::*;

#[test]
fn a_provider_with_no_name_or_command_is_refused() {
    assert!(provider("", &["x"], true).unusable().is_some());
    assert!(provider("v", &[], true).unusable().is_some());
}
