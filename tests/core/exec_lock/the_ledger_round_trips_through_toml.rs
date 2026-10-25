use super::*;

#[test]
fn the_ledger_round_trips_through_toml() {
    let dir = tempfile::tempdir().unwrap();
    let path = ExecLedger::path_in(dir.path());
    let mut l = ExecLedger::new();
    l.record_run("abc", "2026-07-24T00:00:00Z".into(), "./s.sh", None);
    l.save(&path).unwrap();
    let back = ExecLedger::load(&path).unwrap();
    assert_eq!(back, l);
    assert_eq!(back.count("abc"), 1);
}
