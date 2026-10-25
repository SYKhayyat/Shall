use super::*;

/// Already being in the state the line asks for is what convergence *means*, and `sc` says
/// so with an exit code: 1056 to a start of a running service, 1062 to a stop of a stopped
/// one. Per verb, because each is an ordinary failure on the other — a stop that came back
/// "already running" did not stop anything.
#[test]
fn windows_forgives_already_in_that_state_once_per_verb() {
    let sc = shipped("windows-sc");
    assert_eq!(sc.benign_exits(ServiceAction::Start), [1056]);
    assert_eq!(sc.benign_exits(ServiceAction::Stop), [1062]);
    for action in [
        ServiceAction::Enable,
        ServiceAction::Disable,
        ServiceAction::Restart,
    ] {
        assert!(
            sc.benign_exits(action).is_empty(),
            "{:?} asks for a transition, so no code means it already happened",
            action
        );
    }
}
