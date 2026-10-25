use super::*;

/// A caller-supplied zero is a configuration mistake, not a request to block every call for
/// ever. The clamp in `new` is what makes its `expect` unreachable — without it this panics
/// at construction rather than failing a request.
#[tokio::test]
async fn zero_requests_per_minute_is_clamped_rather_than_fatal() {
    let l = RateLimiter::new(0, "zero");
    tokio::time::timeout(Duration::from_secs(5), l.wait())
        .await
        .expect("a limiter clamped from zero would not issue even its first permit")
        .unwrap();
}
