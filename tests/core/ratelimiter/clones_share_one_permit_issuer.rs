use super::*;

/// **The comment on `inner` is a promise, and this is that promise as a test.** It says a
/// per-clone cell "would silently double every limit here" — silently being the problem, so
/// the shape has to be asserted rather than described. Two backends holding clones of one
/// quota must hold ONE issuer.
#[tokio::test]
async fn clones_share_one_permit_issuer() {
    let a = RateLimiter::new(600, "shared");
    let b = a.clone();
    assert!(!a.is_engaged());
    assert!(!b.is_engaged());

    a.wait().await.unwrap();

    assert!(a.is_engaged());
    assert!(
        b.is_engaged(),
        "the clone built an issuer of its own, so the two of them hold twice the quota the \
         caller asked for and nothing says so"
    );
}
