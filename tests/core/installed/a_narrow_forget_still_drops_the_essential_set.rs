use super::*;

/// **A narrow forget is still a forget.** The OS-essential set is the one that refuses
/// removals, so remembering less of it is less safe rather than faster — it goes whole under
/// the narrow scope exactly as it does under the wide one.
#[tokio::test]
async fn a_narrow_forget_still_drops_the_essential_set() {
    let memo = InstalledListings::new();
    let fetches = std::sync::Arc::new(AtomicUsize::new(0));
    let counting = || {
        let fetches = fetches.clone();
        async move {
            fetches.fetch_add(1, Ordering::SeqCst);
            Ok(vec!["apt".to_string()])
        }
    };

    memo.essential_once("brew", counting())
        .await
        .expect("essentials");
    memo.essential_once("apt", counting())
        .await
        .expect("essentials");
    assert_eq!(fetches.load(Ordering::SeqCst), 2);

    memo.forget_manager("apt");

    memo.essential_once("brew", counting())
        .await
        .expect("essentials again");
    assert_eq!(
        fetches.load(Ordering::SeqCst),
        3,
        "a narrow forget kept the essential set, and that set is what refuses removals"
    );
}
