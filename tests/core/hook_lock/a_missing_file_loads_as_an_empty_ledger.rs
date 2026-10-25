use super::*;

#[test]
fn a_missing_file_loads_as_an_empty_ledger() {
    let ledger = HookLedger::load(Path::new("does/not/exist/hooks.toml")).unwrap();
    assert!(ledger.is_empty());
}
