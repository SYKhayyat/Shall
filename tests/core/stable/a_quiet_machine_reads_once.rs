use super::*;

/// The ordinary case, and the one that must cost nothing: nobody is writing, so the first
/// read is the answer.
#[tokio::test]
async fn a_quiet_machine_reads_once() {
    let calls = std::sync::atomic::AtomicUsize::new(0);
    let seen: usize = stable(|| async {
        calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(7)
    })
    .await
    .unwrap();
    assert_eq!(seen, 7);
    assert_eq!(
        calls.load(std::sync::atomic::Ordering::SeqCst),
        1,
        "a reader on a quiet machine must not read anything twice"
    );
}
