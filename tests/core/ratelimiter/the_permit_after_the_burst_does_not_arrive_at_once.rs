use super::*;

/// **The limiter limits, which is the only reason it is a dependency.**
///
/// Asserted by the second permit *not* arriving rather than by waiting a minute for it: at
/// one request per minute the burst is one, so the first permit is immediate and the second
/// cannot be. A build where the quota did nothing passes every other test in this file.
#[tokio::test]
async fn the_permit_after_the_burst_does_not_arrive_at_once() {
    let l = RateLimiter::new(1, "one per minute");
    tokio::time::timeout(Duration::from_millis(500), l.wait())
        .await
        .expect("the first permit was not immediate")
        .unwrap();

    assert!(
        tokio::time::timeout(Duration::from_millis(300), l.wait())
            .await
            .is_err(),
        "a one-per-minute limiter issued two permits inside 300 ms, so the quota is not \
         being enforced at all"
    );
}
