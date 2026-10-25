use super::*;

#[test]
fn windows_restart_is_stop_then_start() {
    let cmds = shipped("windows-sc").plan(ServiceAction::Restart, "W32Time");
    assert_eq!(cmds.len(), 2);
    assert_eq!(cmds[0].1[1], "stop");
    assert_eq!(cmds[1].1[1], "start");
}
