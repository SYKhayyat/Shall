use super::*;

/// A mutation that never returns — the process was killed — leaves the entry `InProgress`,
/// which is the whole point. Modelled by dropping the future rather than awaiting it: the
/// record is written by then, and nothing else runs.
#[tokio::test]
async fn an_abandoned_mutation_stays_interrupted() {
    let tmp = tempdir().unwrap();
    let path = tmp.path().join("j.jsonl");
    let journal = tokio::sync::Mutex::new(Journal::at(path.clone()).unwrap());

    {
        let pending = journalled(
            &journal,
            vec![JournalAction::Install(a_spec("half"))],
            std::future::pending::<Result<()>>(),
        );
        futures::pin_mut!(pending);
        // Polled once — far enough to write the record and start the mutation, never far
        // enough to finish it — then dropped, which is what SIGKILL looks like from here.
        let _ = futures::poll!(&mut pending);
    }

    let after_the_crash = Journal::at(path).unwrap();
    assert_eq!(
        after_the_crash.interrupted_actions().len(),
        1,
        "the next process must find the half-done install recorded"
    );
    assert!(after_the_crash.interrupted_actions()[0]
        .action
        .is_replayable());
}
