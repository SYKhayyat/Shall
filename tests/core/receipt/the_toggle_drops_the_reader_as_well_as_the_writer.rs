use super::*;

/// **`[receipts] enabled = false` is not only about recording.**
///
/// A toggle that governed what is written but not what is believed would let a machine that
/// switched the file off keep claiming ownership from lines an earlier run left behind — the
/// half-setting shape this repo keeps paying to find, and the one `set_receipts_enabled`'s own
/// comment insists on. The file stays on disk and stops being an answer.
#[test]
fn the_toggle_drops_the_reader_as_well_as_the_writer() {
    use shall::core::journal::{Journal, JournalAction};
    use shall::core::PackageSpec;

    let tmp = tempdir().unwrap();
    let mut journal =
        Journal::at(tmp.path().join(Journal::FILE_NAME)).expect("a journal in a temp directory");
    journal.set_receipts_enabled(true);
    let id = journal
        .record_start(JournalAction::Install(PackageSpec {
            name: "jq".into(),
            backend: "apt".into(),
            options: Default::default(),
            requires: vec![],
            present: true,
        }))
        .expect("the WAL opens the operation");
    journal.record_success(&id).expect("the install completes");
    assert!(
        journal.receipt_claims("apt", "jq"),
        "with the toggle on, a recorded install is a claim: the fixture writes no receipt"
    );

    journal.set_receipts_enabled(false);
    assert!(
        !journal.receipt_claims("apt", "jq"),
        "with the toggle off, the same line must stop being a claim"
    );
    assert!(
        journal.receipted_installs().is_empty(),
        "and nothing may be read out of it either"
    );
}
