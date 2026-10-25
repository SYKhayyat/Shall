use super::*;

/// And the other direction, without which the test above passes against a limiter that
/// blocks everything: inside its burst, a generous quota does not make the caller wait.
#[tokio::test]
async fn a_generous_quota_issues_its_burst_without_waiting() {
    let l = RateLimiter::new(600, "600 per minute");
    tokio::time::timeout(Duration::from_secs(2), async {
        for _ in 0..5 {
            l.wait().await.unwrap();
        }
    })
    .await
    .expect("five permits from a 600/minute limiter did not arrive within two seconds");
}
