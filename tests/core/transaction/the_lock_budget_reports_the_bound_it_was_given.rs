use super::*;

/// **The budget reports the bound it was given, not the bound that is left.**
///
/// Found by the mutation gate: `LockBudget::total` replaced with `Default::default()` survived
/// the whole suite. **It is the difference between two sentences about the user's own
/// decision.** The verdict reads it to tell *"the user set 0"* apart from *"the user set 300
/// and Shall spent it"* — and the old message said `manager_lock_wait_secs` is 0 to both, which
/// to someone who set 300 is a sentence about a decision they never made.
///
/// A zero total is the *worst* wrong answer: it says the user asked for no waiting when they
/// asked for five minutes, and it does that in the message that explains a refusal.
#[test]
fn the_lock_budget_reports_the_bound_it_was_given() {
    let budget = LockBudget::of(Duration::from_secs(300));
    assert_eq!(
        budget.total(),
        Duration::from_secs(300),
        "the budget must report the bound it was given, not the bound that is left — the two \
         are different sentences to the user who set it"
    );

    // **And what is left is a different number from the whole**, because the two are read by
    // different parts of the verdict.
    let mut spent = LockBudget::of(Duration::from_secs(300));
    spent.spend(Duration::from_secs(30));
    assert_eq!(spent.total(), Duration::from_secs(300));
    assert_eq!(spent.remaining(), Duration::from_secs(270));
}
