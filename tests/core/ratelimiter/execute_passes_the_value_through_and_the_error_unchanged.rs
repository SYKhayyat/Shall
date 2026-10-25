use super::*;

/// `execute` returns what the closure returned, and hands a failure back unchanged. The
/// `RateLimit` arm logs and must not swallow or reclassify — the class it carries is the
/// one `VI.11` is about.
#[tokio::test]
async fn execute_passes_the_value_through_and_the_error_unchanged() {
    let l = RateLimiter::new(600, "execute");

    let v: u32 = l.execute(|| async { Ok(7u32) }).await.unwrap();
    assert_eq!(v, 7);

    let e = l
        .execute(|| async { Err::<(), _>(Error::RateLimit("429".into())) })
        .await
        .expect_err("execute swallowed a failure");
    assert!(
        matches!(e, Error::RateLimit(_)),
        "execute reclassified the failure it was handed: {e:?}"
    );
}
