use super::*;

/// **The control: an unwind with nothing protected is allowed, and the token says how much it
/// cleared.** Without this, the three refusals above could all pass by a door that refuses
/// everything — which is precisely the mutant the gate caught, because the `!` deleted from
/// `enforce_own_work`'s refusal makes a *clean* unwind the one that is refused.
///
/// **Two packages, so the count is a count.** One would not: the token's count getter replaced
/// with a constant `1` answers the same thing for an unwind of one, and the gate reported that
/// mutation as surviving for exactly that reason. Two packages and an assertion that the token
/// says two is what makes the count a fact rather than a coincidence.
#[tokio::test]
async fn an_unwind_with_nothing_protected_is_allowed() {
    let reg = registry_reporting("apt", &[]);
    let cfg = config_with(0);
    let reaped = enforce_own_work(&cfg, &reg, &pairs(&["htop", "vim"]))
        .await
        .expect("an undo of unprotected packages must be allowed");
    assert_eq!(
        reaped.allowed(),
        2,
        "the token carries what the guard cleared, not a constant"
    );
    assert!(
        reaped.authorises(2),
        "and a token that cleared two removals authorises two removals"
    );
    assert!(
        !reaped.authorises(3),
        "and does not authorise more than it cleared"
    );
}
