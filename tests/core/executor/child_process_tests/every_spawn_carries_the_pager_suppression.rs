use super::*;
use shall::core::executor::DryRunOutput;

/// A pager waits for a keypress nothing will send, and its escape sequences land in the
/// text a parser is about to read. `systemctl status` and `git log` both start one.
#[tokio::test]
async fn every_spawn_carries_the_pager_suppression() {
    let (e, mock) = wired();

    // Stated rather than fallen through (`#162`): what this test asks its manager, and the
    // answer it read from silence before. The assertions here are about the argv.
    mock.set_response(
        "systemctl status -- nginx",
        Ok(DryRunOutput::default().into()),
    );
    let _ = e.run("systemctl", &["status", "--", "nginx"], false).await;
    let env = mock.last_env.lock().await.clone();
    assert_eq!(env.get("SYSTEMD_PAGER").map(String::as_str), Some(""));
    assert_eq!(env.get("PAGER").map(String::as_str), Some("cat"));
    assert_eq!(env.get("GIT_PAGER").map(String::as_str), Some("cat"));
}
