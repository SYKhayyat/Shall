use super::*;

/// A pager waits for a keypress no captured child receives. Every systemctl row has to
/// carry the suppression, not only the two that print a screenful — `list` and `status`
/// are where it was seen, and the rest are the same command deciding the same way.
#[test]
fn every_systemd_row_suppresses_the_pager() {
    let sd = shipped("systemd");
    for action in [
        ServiceAction::Enable,
        ServiceAction::Disable,
        ServiceAction::Start,
        ServiceAction::Stop,
        ServiceAction::Restart,
    ] {
        for (_, cmd) in sd.plan(action, "nginx") {
            assert!(cmd.iter().any(|a| a == "--no-pager"), "{:?} can page", cmd);
        }
    }
    assert!(sd.list.iter().any(|a| a == "--no-pager"), "{:?}", sd.list);
    assert!(
        sd.status.iter().any(|a| a == "--no-pager"),
        "{:?}",
        sd.status
    );
}
