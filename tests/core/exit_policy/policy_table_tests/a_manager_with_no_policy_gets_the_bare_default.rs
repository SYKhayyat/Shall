use super::*;

/// And a name nobody wrote a policy for gets the bare default rather than somebody else's —
/// the control, so the assertion above is not passing because everything answers yes.
#[test]
fn a_manager_with_no_policy_gets_the_bare_default() {
    let p = for_manager("a-manager-nobody-has-written-a-policy-for");
    assert!(!p.reads_output());
    assert!(p.benign_exits.is_empty() && p.transient_exits.is_empty());
}
