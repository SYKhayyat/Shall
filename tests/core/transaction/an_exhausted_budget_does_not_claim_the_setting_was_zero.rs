use super::*;

/// A budget that was SPENT is not a setting of zero. The old message said
/// "`manager_lock_wait_secs` is 0" to someone who set 300 and watched Shall wait out all
/// 300 — a sentence about a decision they never made, and no hint that waiting longer was
/// even possible.
#[test]
fn an_exhausted_budget_does_not_claim_the_setting_was_zero() {
    let verdict = lock_wait_verdict(
        &lock_failure(PACMAN_SAID),
        "pacman",
        Duration::ZERO,
        Duration::from_secs(300),
        &|_| Held::Live("a `pacman`".into()),
    );
    let LockWait::Hopeless(err) = verdict else {
        panic!("nothing left to wait with: {verdict:?}");
    };
    let said = err.to_string();
    assert!(
        said.contains("waited its full 300s"),
        "the message must say Shall spent the budget: {said}"
    );
    assert!(
        !said.contains("`manager_lock_wait_secs` is 0"),
        "and must not claim the setting was zero: {said}"
    );
}
