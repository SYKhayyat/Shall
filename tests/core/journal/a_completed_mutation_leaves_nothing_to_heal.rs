use super::*;

/// Success closes every id, so the entry stops being recovery's business.
#[tokio::test]
async fn a_completed_mutation_leaves_nothing_to_heal() {
    let tmp = tempdir().unwrap();
    let journal = tokio::sync::Mutex::new(Journal::at(tmp.path().join("j.jsonl")).unwrap());

    journalled(
        &journal,
        removals_of("apt", &["a".into(), "b".into()]),
        async { Ok(()) },
    )
    .await
    .unwrap();

    let j = journal.lock().await;
    assert_eq!(
        j.entries.len(),
        2,
        "one entry per name, not one per command"
    );
    assert!(
        !j.needs_recovery(),
        "a mutation that finished is not interrupted work"
    );
}
