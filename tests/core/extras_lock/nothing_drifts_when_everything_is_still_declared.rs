use super::*;

#[test]
fn nothing_drifts_when_everything_is_still_declared() {
    let mut ledger = ExtrasLedger::new();
    ledger.record(records(&["shim:rg"]));
    assert!(drifted(&ledger, &["shim:rg"]).is_empty());
}
