use super::*;

/// A derived restart's two halves carry their own verbs, so each forgives its own code. A
/// single per-provider list could not tell them apart, and would let a failed stop through.
#[test]
fn a_derived_restart_labels_each_half_with_its_own_verb() {
    let sc = shipped("windows-sc");
    let cmds = sc.plan(ServiceAction::Restart, "W32Time");
    assert_eq!(cmds[0].0, ServiceAction::Stop);
    assert_eq!(cmds[1].0, ServiceAction::Start);
    assert_eq!(sc.benign_exits(cmds[0].0), [1062]);
    assert_eq!(sc.benign_exits(cmds[1].0), [1056]);
}
