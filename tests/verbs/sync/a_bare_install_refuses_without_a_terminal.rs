use super::*;

#[test]
fn a_bare_install_refuses_without_a_terminal() {
    let mut changes = shall::app::sync::planner::SyncChanges::default();
    changes.add_installs(&[spec(true)]);
    let error = confirm_bare_installs(&changes, false).expect_err("stdin is not a terminal");
    assert!(error.to_string().contains("--yes"), "{error}");
}
