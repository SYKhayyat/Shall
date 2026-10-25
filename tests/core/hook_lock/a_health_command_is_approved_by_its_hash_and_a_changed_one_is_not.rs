use super::*;

#[test]
fn a_health_command_is_approved_by_its_hash_and_a_changed_one_is_not() {
    // U31: a health-check command rides the ledger. Approving the exact command makes it
    // Approved; editing it one byte makes it an unapproved (New) check, never a silent pass.
    let cmd = "systemctl is-active nginx";
    let mut ledger = HookLedger::new();
    ledger.approve(&health_id(cmd), &hash_script(cmd));
    assert!(ledger
        .verdict(&health_id(cmd), &hash_script(cmd))
        .is_approved());
    let edited = "systemctl is-active nginx2";
    assert!(!ledger
        .verdict(&health_id(edited), &hash_script(edited))
        .is_approved());
}
