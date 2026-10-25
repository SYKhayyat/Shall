use super::*;

#[test]
fn re_approving_updates_to_the_new_hash() {
    // `shall lock` after a reviewed change: the new hash becomes the approved one.
    let mut ledger = HookLedger::new();
    ledger.approve("after_install:nginx", "old");
    ledger.approve("after_install:nginx", "new");
    assert_eq!(
        ledger.verdict("after_install:nginx", "new"),
        Verdict::Approved
    );
    assert!(matches!(
        ledger.verdict("after_install:nginx", "old"),
        Verdict::Changed { .. }
    ));
}
