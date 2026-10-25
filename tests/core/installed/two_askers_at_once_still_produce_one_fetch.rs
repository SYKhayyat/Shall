use super::*;

/// **And the dedup the long lock hold exists for still works.**
///
/// The wrong fix for the test above is to shorten the hold, and the module's own comment
/// says so: two concurrent askers must produce **one** subprocess, not two. This asserts
/// the property that would be lost, so a generation counter cannot be traded for it.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn two_askers_at_once_still_produce_one_fetch() {
    let memo = std::sync::Arc::new(InstalledListings::new());
    let fetches = std::sync::Arc::new(AtomicUsize::new(0));

    let ask = |memo: std::sync::Arc<InstalledListings>, fetches: std::sync::Arc<AtomicUsize>| async move {
        memo.once("test", async move {
            fetches.fetch_add(1, Ordering::SeqCst);
            // The duration that makes the race real. Without it both callers finish before
            // either could have contended, which is why nothing in the suite has ever
            // exercised this.
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            Ok(vec![pkg("one")])
        })
        .await
        .expect("a listing")
    };

    let (a, b) = tokio::join!(
        ask(memo.clone(), fetches.clone()),
        ask(memo.clone(), fetches.clone())
    );
    assert_eq!(a.len(), 1);
    assert_eq!(b.len(), 1);
    assert_eq!(
        fetches.load(Ordering::SeqCst),
        1,
        "two concurrent askers ran two subprocesses. The slot lock must be held across the \
         fetch — an install of forty plugins must not launch forty listings."
    );
}
