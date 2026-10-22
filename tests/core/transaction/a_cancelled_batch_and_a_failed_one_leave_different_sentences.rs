use super::*;

/// **A mutant found this, not a person.** The choice was a match guard inside the journal
/// loop, and the nightly shard replaced it with `true` and with `false` - both survived. The
/// two sentences are different facts to whoever reads the WAL afterwards, so both are pinned.
#[test]
fn a_cancelled_batch_and_a_failed_one_leave_different_sentences() {
    let e = Error::Transaction("`apt` failed (exit 100)".into());
    let cancelled = wal_failure_reason(true, &e);
    let failed = wal_failure_reason(false, &e);
    assert!(
        cancelled.contains("cancelled before this batch ran"),
        "a cancelled batch must say so: {cancelled}"
    );
    assert!(
        !cancelled.contains("exit 100"),
        "a cancelled batch never ran, so quoting the error blames it for something it did \
         not do: {cancelled}"
    );
    assert!(
        failed.contains("exit 100"),
        "a failure must carry the manager's own words: {failed}"
    );
    assert_ne!(
        cancelled, failed,
        "the guard decides nothing if both read the same"
    );
}
