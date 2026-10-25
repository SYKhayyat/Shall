use super::*;

#[test]
fn a_qualified_install_does_not_prompt() {
    let mut changes = shall::app::sync::planner::SyncChanges::default();
    changes.add_installs(&[spec(false)]);
    let mut asked = false;
    assert!(confirm_bare_installs_with(&changes, false, |_| {
        asked = true;
        Ok(true)
    })
    .unwrap());
    assert!(!asked);
}
