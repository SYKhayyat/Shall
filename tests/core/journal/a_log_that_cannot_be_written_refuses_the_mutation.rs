use super::*;

/// A WAL write that fails stops the mutation instead of letting it run unrecorded — the
/// same call the transaction engine makes stillborn. Provoked by pointing the log at a
/// path whose parent is a file, so the append cannot succeed.
#[tokio::test]
async fn a_log_that_cannot_be_written_refuses_the_mutation() {
    let tmp = tempdir().unwrap();
    let blocker = tmp.path().join("not-a-dir");
    std::fs::write(&blocker, b"x").unwrap();
    let journal =
        tokio::sync::Mutex::new(Journal::at(blocker.join("sub").join("j.jsonl")).unwrap());

    let ran = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let flag = ran.clone();
    let outcome: Result<()> = journalled(
        &journal,
        vec![JournalAction::Install(a_spec("htop"))],
        async move {
            flag.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        },
    )
    .await;

    assert!(outcome.is_err(), "an unrecordable mutation must not run");
    assert!(
        !ran.load(std::sync::atomic::Ordering::SeqCst),
        "the manager was invoked with nothing recording that it had been"
    );
}
