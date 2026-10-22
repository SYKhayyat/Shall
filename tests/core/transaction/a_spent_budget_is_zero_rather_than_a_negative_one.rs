use super::*;

/// A budget that has been overrun is spent, not negative.
///
/// `lock_wait_verdict` reads a zero wait as "the user opted out", which is the right answer
/// once there is nothing left â€” and it is reached by saturating rather than by a subtraction
/// that would panic on the way past.
#[test]
fn a_spent_budget_is_zero_rather_than_a_negative_one() {
    let mut budget = LockBudget::of(Duration::from_secs(30));
    budget.spend(Duration::from_secs(45));
    assert_eq!(budget.remaining(), Duration::ZERO);
}
