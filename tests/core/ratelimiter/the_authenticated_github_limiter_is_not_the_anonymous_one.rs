use super::*;

/// **A token changes the local budget as well as the remote one**, which is why handing the
/// harness containers a `GITHUB_TOKEN` turned the `github:` lifecycle from a rate-limit
/// casualty into an ordinary install: 1 request per minute against 80.
#[tokio::test]
async fn the_authenticated_github_limiter_is_not_the_anonymous_one() {
    assert_eq!(
        RateLimiter::github().description(),
        "GitHub (Unauthenticated)"
    );
    assert_eq!(
        RateLimiter::github_authenticated().description(),
        "GitHub (Authenticated)"
    );

    // Behaviour, not the label. The authenticated quota issues a burst the other cannot.
    let auth = RateLimiter::github_authenticated();
    tokio::time::timeout(Duration::from_secs(2), async {
        for _ in 0..5 {
            auth.wait().await.unwrap();
        }
    })
    .await
    .expect("the authenticated limiter stalled inside its own burst");

    let anon = RateLimiter::github();
    anon.wait().await.unwrap();
    assert!(
        tokio::time::timeout(Duration::from_millis(300), anon.wait())
            .await
            .is_err(),
        "the anonymous limiter issued a second permit at once, so the two are the same \
         budget and the token buys nothing locally"
    );
}
