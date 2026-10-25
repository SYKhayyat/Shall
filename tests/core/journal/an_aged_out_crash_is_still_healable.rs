use super::*;

#[test]
fn an_aged_out_crash_is_still_healable() {
    // R23: `cleanup` flips an InProgress entry to Abandoned after 4h. That must not take
    // it out of heal's reach -- a node aborted mid-removal never entered the rollback
    // history, so the WAL is the only thing that knows the package is half-removed.
    let tmp = tempdir().unwrap();
    let mut journal = Journal::at(tmp.path().join("journal.json")).unwrap();

    let id = journal
        .record_start(JournalAction::Remove {
            name: "python3".into(),
            backend: "apt".into(),
        })
        .unwrap();

    // Backdate past the 4h staleness limit, then age it out.
    journal.entries.get_mut(&id).unwrap().started_at_unix =
        (Utc::now() - ChronoDuration::hours(5)).timestamp();
    journal.cleanup().unwrap();

    assert_eq!(
        journal.entries[&id].status,
        ActionStatus::Abandoned,
        "cleanup should still age the entry out"
    );
    assert!(
        journal.needs_recovery(),
        "an abandoned mutation must still trigger a heal"
    );
    assert!(
        journal.interrupted_actions().iter().any(|e| e.id == id),
        "an abandoned mutation must still be offered to heal"
    );
}
