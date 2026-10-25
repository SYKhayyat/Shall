use super::*;

/// **A wait for a lock that is not held ends at once**, so the settle step `heal` runs costs
/// nothing on the ordinary machine — which is every machine that has not just been killed
/// mid-transaction. One poll interval, not the budget.
#[tokio::test]
async fn a_lock_nobody_holds_is_not_waited_for() {
    let started = std::time::Instant::now();
    let out = wait_until_not_held("pacman", std::time::Duration::from_secs(30), &|| false).await;
    assert!(matches!(out, Waited::Freed(_)), "{out:?}");
    assert!(
        started.elapsed() < std::time::Duration::from_secs(5),
        "a free lock took {:?} to answer",
        started.elapsed()
    );
}
