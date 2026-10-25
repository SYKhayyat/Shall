use super::*;

#[test]
fn an_unrecorded_hook_is_new_not_trusted() {
    let ledger = HookLedger::new();
    assert_eq!(ledger.verdict("after_install:nginx", "abc"), Verdict::New);
}
