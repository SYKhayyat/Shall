use super::*;

#[test]
fn drift_is_recorded_minus_declared() {
    let mut ledger = ExtrasLedger::new();
    ledger.record(records(&["service:nginx", "shim:rg", "repo:apt:ppa:x/y"]));
    // The user deleted the service line; the other two remain.
    assert_eq!(
        drifted(&ledger, &["shim:rg", "repo:apt:ppa:x/y"]),
        vec!["service:nginx".to_string()]
    );
}
