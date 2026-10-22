use super::*;

/// The primitive both async doors are built on. It hands the blocking wait to a pool rather
/// than the runtime, so the assertion worth making is that it still comes back holding the
/// lock rather than merely coming back.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_off_runtime_wait_returns_holding_the_lock() {
    let _g = TEST_GATE.lock().await;
    let dir = tmp("acquire-async");

    let taken = DataLock::acquire_async(&dir, "sync", Duration::from_secs(5))
        .await
        .unwrap();
    assert!(
        held(),
        "acquire_async returned without the lock it promised"
    );
    assert!(
        lock_file_is_in(&dir),
        "acquire_async locked a directory other than the one it was handed"
    );
    drop(taken);
    assert!(!held());
}
