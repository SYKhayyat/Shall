use super::*;

/// **An invalidation reaches a task that was already waiting.**
///
/// R2, and it needs duration to exist. `once` clones the `Arc<Slot>` out of the `DashMap`
/// *before* it waits on that slot's mutex, and it waits there for the length of a real
/// listing subprocess — measured elsewhere in this tree at over a second on Windows. The
/// long hold is deliberate and correct: it is what makes two askers produce one
/// `winget list`. But `forget_all` clears the *map*, and a map entry dropped after the
/// `Arc` was handed out invalidates nothing for whoever is holding it. The waiter then won
/// the lock and returned a listing that predated both the install and the invalidation
/// meant to cover it — a needless reinstall, or a drift report naming a package that was
/// just fixed.
///
/// The fix is the one already in this tree, in `resolver.rs`: put the generation in the
/// answer, so an invalidated entry cannot be reached by an already-cloned handle.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_listing_taken_before_an_invalidation_is_not_served_after_it() {
    let memo = std::sync::Arc::new(InstalledListings::new());
    let fetches = std::sync::Arc::new(AtomicUsize::new(0));

    // Round one: somebody asks, and the answer is remembered.
    let first = memo
        .once("test", {
            let fetches = fetches.clone();
            async move {
                fetches.fetch_add(1, Ordering::SeqCst);
                Ok(vec![pkg("before")])
            }
        })
        .await
        .expect("the first listing");
    assert_eq!(first.len(), 1);
    assert_eq!(fetches.load(Ordering::SeqCst), 1);

    // A mutation happens. `CommandExecutor::run` calls exactly this after one finishes.
    memo.forget_all();

    // The waiter's shape: a task holding a handle from before the clear. It cannot be
    // served the old answer, whatever it is still holding.
    let second = memo
        .once("test", {
            let fetches = fetches.clone();
            async move {
                fetches.fetch_add(1, Ordering::SeqCst);
                Ok(vec![pkg("after"), pkg("also-after")])
            }
        })
        .await
        .expect("the second listing");
    assert_eq!(
        fetches.load(Ordering::SeqCst),
        2,
        "the memo answered from a listing taken before the invalidation; clearing the map \
         does not reach an `Arc` already cloned out of it"
    );
    assert_eq!(
        second.len(),
        2,
        "the post-mutation listing was not the one returned"
    );
}
