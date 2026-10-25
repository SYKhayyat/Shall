use super::*;

/// A failed attempt is not interrupted work (owner ruling, 2026-08-05 — `Q33`).
///
/// It reached an outcome and said so; the package is not installed and its line is still in
/// the manifest, so the next `sync` schedules it again. Recovering it here was the same work
/// twice — and it compounded, because one interrupted entry that can never be recovered
/// keeps `needs_recovery` true for ever, so every sync ran a full recovery of every failure
/// the machine had ever recorded in front of it. 208 seconds of one `watch --once`.
#[test]
fn recovery_finishes_interrupted_work_and_does_not_retry_failures() {
    let tmp = tempdir().unwrap();
    let mut journal = Journal::at(tmp.path().join("journal.json")).unwrap();

    let spec = |name: &str| shall::core::PackageSpec {
        name: name.to_string(),
        backend: "apt".to_string(),
        options: Default::default(),
        requires: Vec::new(),
        present: true,
    };
    let interrupted = journal
        .record_start(JournalAction::Install(spec("half-done")))
        .unwrap();
    let failed = journal
        .record_start(JournalAction::Install(spec("typo")))
        .unwrap();
    journal.record_failure(&failed, "no such package").unwrap();
    let done = journal
        .record_start(JournalAction::Install(spec("fine")))
        .unwrap();
    journal.record_success(&done).unwrap();

    let offered: Vec<String> = journal
        .interrupted_actions()
        .into_iter()
        .map(|e| e.id)
        .collect();
    assert_eq!(
        offered,
        vec![interrupted.clone()],
        "only the entry that started and never reached an outcome is recovery's to finish"
    );
    assert!(
        journal.needs_recovery(),
        "the trigger and the work must be the same predicate — when they disagreed, one \
         interrupted entry pulled every past failure along with it"
    );

    // And with the interrupted one resolved, nothing runs at all.
    journal.record_success(&interrupted).unwrap();
    assert!(!journal.needs_recovery());
    assert!(journal.interrupted_actions().is_empty());
}
