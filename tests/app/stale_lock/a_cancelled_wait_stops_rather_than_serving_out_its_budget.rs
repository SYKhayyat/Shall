use super::*;

/// A cancelled run stops waiting rather than serving out its budget: five minutes after a
/// Ctrl-C is five minutes of looking hung, and a hang is what gets killed.
#[tokio::test]
async fn a_cancelled_wait_stops_rather_than_serving_out_its_budget() {
    let out = wait_until_not_held("pacman", std::time::Duration::from_secs(300), &|| true).await;
    assert_eq!(out, Waited::Cancelled);
}
