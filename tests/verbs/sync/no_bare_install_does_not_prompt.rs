use super::*;

#[test]
fn no_bare_install_does_not_prompt() {
    let changes = shall::app::sync::planner::SyncChanges::default();
    let mut asked = false;
    assert!(confirm_bare_installs_with(&changes, false, |_| {
        asked = true;
        Ok(false)
    })
    .unwrap());
    assert!(!asked);
}
