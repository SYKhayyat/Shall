use super::*;

/// **A manager that is not on this machine is not the reason nothing happens.** The planner
/// already says "`pacman` is not on this machine"; answering "`pacman` cannot pin" instead
/// would replace a true sentence with a misleading one, and would fire on every host for
/// every declaration meant for a different one.
#[test]
fn a_manager_that_is_not_here_raises_nothing() {
    let reg = registry(&[("cannot", false, false)]);
    assert!(found(&reg, &[spec("jq", "cannot", Some("1.7"))]).is_empty());
}
