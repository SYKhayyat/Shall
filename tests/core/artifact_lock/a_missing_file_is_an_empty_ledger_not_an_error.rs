use super::*;

#[test]
fn a_missing_file_is_an_empty_ledger_not_an_error() {
    let dir = TempDir::new().unwrap();
    let led = ArtifactLedger::load(&dir.path().join("github.toml")).unwrap();
    assert!(led.is_empty());
}
