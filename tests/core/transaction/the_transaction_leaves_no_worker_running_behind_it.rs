use super::*;

/// **The transaction stops its workers before it finishes, and waits for them to be gone.**
///
/// Found by the mutation gate: `Transaction::quiesce_workers` replaced with `()` survived the
/// whole suite. **It is the door between a finished run and a still-running one.** Quiescing
/// is how the transaction stops the background workers it started before it goes on to decide
/// what to undo — `abort_all` only *requests* cancellation, and the dropped worker's guard
/// sends SIGTERM and, being a `Drop`, cannot stay to watch. A manager takes up to the
/// termination grace after that signal to release its package database, and **every command the
/// rollback is about to issue competes for exactly those locks** — so a rollback started without
/// the wait fails on wounds the engine itself inflicted.
///
/// **A transaction that quits without quiescing can leave the WAL, the registry and the machine
/// disagreeing**, which is the crash-window family this file exists to handle.
#[tokio::test]
async fn the_transaction_leaves_no_worker_running_behind_it() {
    let mut pool: tokio::task::JoinSet<Vec<TaskResult>> = tokio::task::JoinSet::new();
    pool.spawn(async {
        // A worker that would still be running a minute later, if nobody stopped it.
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        Vec::new()
    });
    assert!(
        !pool.is_empty(),
        "the worker never started, so nothing below means anything"
    );

    Transaction::quiesce_workers(&mut pool).await;

    assert!(
        pool.is_empty(),
        "a worker was still running after the transaction quiesced: the rollback would then \
         compete for the very package-database locks the dying worker is holding"
    );
}
