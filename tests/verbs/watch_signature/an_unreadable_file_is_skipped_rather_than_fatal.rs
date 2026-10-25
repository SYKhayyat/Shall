use super::*;

/// A file that cannot be read is skipped, not fatal: a watcher that dies on a permission
/// error is a watcher that stopped watching.
#[tokio::test]
async fn an_unreadable_file_is_skipped_rather_than_fatal() {
    let dir = repo().await;
    let sig = signature(dir.path()).await;
    assert!(!sig.contains_key("modules/does-not-exist.txt"));
    assert!(sig.contains_key("modules/dev.txt"));
}
