use super::*;

#[test]
fn launchd_restart_is_stop_then_start() {
    let cmds = shipped("launchd").plan(ServiceAction::Restart, "com.foo");
    assert_eq!(cmds.len(), 2);
}
