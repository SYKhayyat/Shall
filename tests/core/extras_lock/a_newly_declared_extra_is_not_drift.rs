use super::*;

#[test]
fn a_newly_declared_extra_is_not_drift() {
    // A key declared now but not in the ledger is an ADD, not a removal — apply handles
    // it; drift() must not report it.
    let ledger = ExtrasLedger::new();
    assert!(drifted(&ledger, &["shim:new"]).is_empty());
}
