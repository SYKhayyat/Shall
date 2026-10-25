use super::*;

#[test]
fn a_row_with_no_recorded_path_is_never_read_as_departed() {
    use std::collections::BTreeSet;
    let mut l = ExecLedger::new();
    l.record_run("h", "t".into(), "./s.sh", None);
    // Simulate a row written before paths were kept.
    l.forget("h");
    let mut bare = ExecLedger::new();
    bare.runs.insert(
        "h".into(),
        ExecRecord {
            count: 1,
            ..Default::default()
        },
    );
    assert!(bare.departed(&BTreeSet::new()).is_empty());
}
