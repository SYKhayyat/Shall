use super::*;

/// **`essential()` is memoised through the same seam, and invalidated by the same call.**
///
/// I7: a live subprocess per backend, on every removal path, asked at six call sites, so one
/// `sync` with removals ran the whole set at least twice and a rollback three times. Routed
/// through `InstalledListings` rather than memoised locally, so it inherits this
/// invalidation instead of inventing a second policy — including the generation counter, so
/// the new memo does not inherit R2's staleness window.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn the_essential_set_is_asked_once_and_forgotten_with_the_listings() {
    let memo = InstalledListings::new();
    let asks = std::sync::Arc::new(AtomicUsize::new(0));

    for _ in 0..3 {
        let set = memo
            .essential_once("test", {
                let asks = asks.clone();
                async move {
                    asks.fetch_add(1, Ordering::SeqCst);
                    Ok(vec!["libc".to_string()])
                }
            })
            .await
            .expect("the essential set");
        assert_eq!(set.len(), 1);
    }
    assert_eq!(
        asks.load(Ordering::SeqCst),
        1,
        "the essential query ran once per asker. It is a subprocess per backend and it \
         cannot change while nothing is being installed."
    );

    memo.forget_all();
    let _ = memo
        .essential_once("test", {
            let asks = asks.clone();
            async move {
                asks.fetch_add(1, Ordering::SeqCst);
                Ok(vec!["libc".to_string(), "coreutils".to_string()])
            }
        })
        .await
        .expect("the essential set again");
    assert_eq!(
        asks.load(Ordering::SeqCst),
        2,
        "a mutation did not invalidate the essential set. A removal guard answering from \
         before the machine changed is the one place a stale answer is not advisory."
    );
}
