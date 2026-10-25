use super::*;

/// The property the whole thing rests on, and the one an ordinary "did it write an entry"
/// test would pass without: the record is on disk **before** the manager is invoked.
///
/// A wrapper that recorded around the call and flushed afterwards would satisfy every
/// assertion about the finished state and provide no recovery at all, because the case it
/// exists for is the process dying in the middle. So the mutation itself is the observer —
/// it reads the log from a second handle on the same file, which is what a fresh process
/// after a crash would see.
#[tokio::test]
async fn the_record_reaches_disk_before_the_mutation_runs() {
    let tmp = tempdir().unwrap();
    let path = tmp.path().join("journal.jsonl");
    let journal = tokio::sync::Mutex::new(Journal::at(path.clone()).unwrap());

    let seen_by_a_later_process = {
        let path = path.clone();
        journalled(
            &journal,
            vec![JournalAction::Install(a_spec("htop"))],
            async move { Journal::at(path).map(|j| j.interrupted_actions().len()) },
        )
        .await
        .unwrap()
    };

    assert_eq!(
        seen_by_a_later_process, 1,
        "a process starting while the mutation was running saw no interrupted entry — the \
         log is being written after the fact, which is not a write-ahead log"
    );
}
