use super::*;

/// An error from the read is the caller's error, not a reason to try again — retrying a
/// failing read three times turns one message into three seconds of nothing.
#[tokio::test]
async fn a_failed_read_is_not_retried() {
    let calls = std::sync::atomic::AtomicUsize::new(0);
    let outcome: Result<()> = stable(|| async {
        calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Err(shall::core::Error::Other("the file is gibberish".into()))
    })
    .await;
    assert!(outcome.is_err());
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1);
}
