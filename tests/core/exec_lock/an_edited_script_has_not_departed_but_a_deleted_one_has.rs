use super::*;

/// The distinction the whole teardown rests on: an edited script is not a departed one.
/// Keyed by hash, an edit reads as a removal and runs the undo of a line that is still
/// declared — the enrol script un-enrolling itself because a typo was fixed.
#[test]
fn an_edited_script_has_not_departed_but_a_deleted_one_has() {
    use std::collections::BTreeSet;
    let mut l = ExecLedger::new();
    l.record_run("hash-v1", "t".into(), "./enrol.sh", Some("echo undo"));
    l.record_run("hash-v2", "t".into(), "./enrol.sh", Some("echo undo"));
    l.record_run("hash-x", "t".into(), "./gone.sh", None);

    // `./enrol.sh` is still declared (its content changed); `./gone.sh` is not.
    let declared: BTreeSet<String> = ["./enrol.sh".to_string()].into_iter().collect();
    let departed = l.departed(&declared);
    assert_eq!(departed.len(), 1, "{:?}", departed);
    assert_eq!(departed[0].1.script.as_deref(), Some("./gone.sh"));

    // With nothing declared at all, every row has departed — deleting the last line is
    // still a removal.
    assert_eq!(l.departed(&BTreeSet::new()).len(), 3);
}
