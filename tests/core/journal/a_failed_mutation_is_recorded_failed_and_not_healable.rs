use super::*;

/// A failure is terminal and reported, not interrupted (`Q33`) — so it must not leave the
/// next sync running a recovery of something that already told the user what happened.
#[tokio::test]
async fn a_failed_mutation_is_recorded_failed_and_not_healable() {
    let tmp = tempdir().unwrap();
    let journal = tokio::sync::Mutex::new(Journal::at(tmp.path().join("j.jsonl")).unwrap());

    let outcome: Result<()> = journalled(
        &journal,
        vec![JournalAction::Remove {
            name: "nope".into(),
            backend: "apt".into(),
        }],
        async { Err(Error::Other("no such package".into())) },
    )
    .await;

    assert!(outcome.is_err(), "the caller's error must reach the caller");
    let j = journal.lock().await;
    let entry = j.entries.values().next().unwrap();
    assert_eq!(entry.status, ActionStatus::Failed);
    assert!(entry.error.as_deref().unwrap().contains("no such package"));
    assert!(!j.needs_recovery());
}
