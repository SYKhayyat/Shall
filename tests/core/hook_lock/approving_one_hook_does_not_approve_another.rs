use super::*;

#[test]
fn approving_one_hook_does_not_approve_another() {
    // Distinct identities: approving nginx's hook must not vouch for redis's.
    let mut ledger = HookLedger::new();
    ledger.approve(&hook_id("after_install", "nginx"), "hash-n");
    assert!(ledger
        .verdict(&hook_id("after_install", "nginx"), "hash-n")
        .is_approved());
    assert_eq!(
        ledger.verdict(&hook_id("after_install", "redis"), "hash-r"),
        Verdict::New
    );
}
