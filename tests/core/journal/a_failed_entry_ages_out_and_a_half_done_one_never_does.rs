use super::*;

/// A `Failed` entry is terminal, so it ages out like any other terminal entry. Kept for ever
/// it would have traded an unbounded retry for an unbounded file: nothing else in the program
/// reads one, and every recovery attempt writes a fresh one.
///
/// `InProgress` is the one status that is never purged at any age — it is the only record
/// that something on this machine is half-done.
#[test]
fn a_failed_entry_ages_out_and_a_half_done_one_never_does() {
    let tmp = tempdir().unwrap();
    let mut journal = Journal::at(tmp.path().join("journal.json")).unwrap();

    let spec = |name: &str| shall::core::PackageSpec {
        name: name.to_string(),
        backend: "apt".to_string(),
        options: Default::default(),
        requires: Vec::new(),
        present: true,
    };
    let failed = journal
        .record_start(JournalAction::Install(spec("typo")))
        .unwrap();
    journal.record_failure(&failed, "no such package").unwrap();
    let completed = journal
        .record_start(JournalAction::Install(spec("fine")))
        .unwrap();
    journal.record_success(&completed).unwrap();
    let half_done = journal
        .record_start(JournalAction::Install(spec("half-done")))
        .unwrap();

    let ancient = (Utc::now() - ChronoDuration::days(30)).timestamp();
    for id in [&failed, &completed, &half_done] {
        let e = journal.entries.get_mut(id).unwrap();
        e.started_at_unix = ancient;
        e.finished_at_unix = Some(ancient);
    }

    assert!(journal.cleanup_expired_logs(7).unwrap());
    assert!(
        !journal.entries.contains_key(&failed),
        "a failed attempt is terminal — nothing reads it and heal no longer retries it"
    );
    assert!(!journal.entries.contains_key(&completed));
    assert!(
        journal.entries.contains_key(&half_done),
        "an unresolved InProgress entry is the only record that a package is half-installed"
    );
}
