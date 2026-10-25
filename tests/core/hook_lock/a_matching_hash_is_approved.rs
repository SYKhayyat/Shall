use super::*;

#[test]
fn a_matching_hash_is_approved() {
    let mut ledger = HookLedger::new();
    let h = hash_script("./setup.sh");
    ledger.approve("after_install:nginx", &h);
    assert_eq!(ledger.verdict("after_install:nginx", &h), Verdict::Approved);
    assert!(ledger.verdict("after_install:nginx", &h).is_approved());
}
