use super::*;

#[test]
fn a_drifted_script_is_changed_and_carries_both_hashes() {
    let mut ledger = HookLedger::new();
    let old = hash_script("./setup.sh v1");
    let new = hash_script("./setup.sh v2 — now with curl | sh");
    ledger.approve("after_install:nginx", &old);
    match ledger.verdict("after_install:nginx", &new) {
        Verdict::Changed { was, now } => {
            assert_eq!(was, old);
            assert_eq!(now, new);
        }
        other => panic!("expected Changed, got {:?}", other),
    }
}
