use super::*;

/// The wait ends rather than running to its deadline when the run is cancelled â€” a Ctrl-C
/// during a five-minute wait must not become a five-minute wait.
#[tokio::test]
async fn a_cancelled_run_stops_waiting_immediately() {
    let token = CancellationToken::new();
    token.cancel();
    let out = wait_for_manager_lock("pacman", "a `pacman`", Duration::from_secs(300), &token)
        .await
        .expect_err("a cancelled wait does not succeed");
    assert!(matches!(out, Error::Cancelled), "{out:?}");
}
