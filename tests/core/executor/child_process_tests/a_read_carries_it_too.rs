use super::*;
use shall::core::executor::DryRunOutput;

/// The suppression must not be a property of the mutating path only — a read is exactly
/// where a pager's banner corrupts the parse.
#[tokio::test]
async fn a_read_carries_it_too() {
    let (e, mock) = wired();

    // Stated rather than fallen through (`#162`): what this test asks its manager, and the
    // answer it read from silence before. The assertions here are about the argv.
    mock.set_response("git log --oneline", Ok(DryRunOutput::default().into()));
    let _ = e.run_output("git", &["log", "--oneline"], false).await;
    let env = mock.last_env.lock().await.clone();
    assert_eq!(env.get("GIT_PAGER").map(String::as_str), Some("cat"));
    assert!(env.contains_key(super::INSIDE_SHALL));
}
