use super::*;

/// A rewrite with the *same bytes* is not a change: that is what makes a `git pull` that
/// brought nothing new a non-event rather than a tick.
#[tokio::test]
async fn the_same_bytes_rewritten_is_not_a_change() {
    let dir = repo().await;
    let before = signature(dir.path()).await;
    std::fs::write(dir.path().join("modules/dev.txt"), "apt:htop\n").expect("the same bytes");
    assert!(changed(&before, &signature(dir.path()).await).is_empty());
}
