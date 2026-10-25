use super::*;

#[test]
fn explicit_confirmation_skips_the_bare_prompt() {
    let mut changes = shall::app::sync::planner::SyncChanges::default();
    changes.add_installs(&[spec(true)]);
    let mut asked = false;
    assert!(confirm_bare_installs_with(&changes, true, |_| {
        asked = true;
        Ok(false)
    })
    .unwrap());
    assert!(!asked);
}
