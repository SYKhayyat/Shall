use super::*;

#[test]
fn a_missing_file_loads_as_an_empty_ledger() {
    assert!(ExecLedger::load(Path::new("does/not/exist/exec.toml"))
        .unwrap()
        .is_empty());
}
