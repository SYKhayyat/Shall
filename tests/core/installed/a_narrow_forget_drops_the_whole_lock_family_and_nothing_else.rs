use super::*;

/// **The narrowed forget, and the family it narrows by.** `pacman` and `yay` are two
/// backends over one manager lock, so an install through either must drop both listings —
/// and a backend that shares no lock with it must keep its own (`PLAN.md` #79).
#[tokio::test]
async fn a_narrow_forget_drops_the_whole_lock_family_and_nothing_else() {
    let memo = InstalledListings::new();
    let fetches = std::sync::Arc::new(AtomicUsize::new(0));
    let counting = |name: &'static str| {
        let fetches = fetches.clone();
        async move {
            fetches.fetch_add(1, Ordering::SeqCst);
            Ok(vec![pkg(name)])
        }
    };

    for backend in ["pacman", "yay", "brew"] {
        memo.once(backend, counting(backend))
            .await
            .expect("first listing");
    }
    assert_eq!(fetches.load(Ordering::SeqCst), 3);

    memo.forget_manager("pacman");

    // The family is asked again: `yay` shares `pacman`'s lock, so it is not safe to keep.
    memo.once("pacman", counting("pacman"))
        .await
        .expect("pacman again");
    memo.once("yay", counting("yay")).await.expect("yay again");
    assert_eq!(
        fetches.load(Ordering::SeqCst),
        5,
        "a narrow forget through `pacman` left a listing in its own lock family standing"
    );

    // And the backend that shares nothing with it is still answered from memory, which is
    // the entire reason the setting exists.
    memo.once("brew", counting("brew"))
        .await
        .expect("brew again");
    assert_eq!(
        fetches.load(Ordering::SeqCst),
        5,
        "a narrow forget re-listed `brew`, which no mutation of `pacman` could have changed"
    );
}
