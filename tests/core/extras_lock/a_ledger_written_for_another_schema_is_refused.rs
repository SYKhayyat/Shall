use super::*;

#[test]
fn a_ledger_written_for_another_schema_is_refused() {
    let err = toml::from_str::<ExtrasLedger>("schema = 3\n").expect_err("refused");
    assert!(err.to_string().contains("schema 3"), "{err}");
}
