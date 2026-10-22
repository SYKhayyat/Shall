use super::*;

/// Two waits in a row cost the setting once between them.
///
/// The value that made this worth extracting: reaching the running total in the engine needs
/// a second package manager holding a real lock and letting go of it twice, which no
/// hermetic test can arrange. Every intermediate value is asserted, because a budget that
/// counted *down* from zero and a budget that never moved both leave the same final answer
/// once it saturates.
#[test]
fn the_wait_budget_is_spent_across_attempts_and_not_per_attempt() {
    let mut budget = LockBudget::of(Duration::from_secs(300));
    assert_eq!(budget.remaining(), Duration::from_secs(300));

    budget.spend(Duration::from_secs(120));
    assert_eq!(
        budget.remaining(),
        Duration::from_secs(180),
        "the second wait gets what the first one left, not the whole setting again"
    );

    budget.spend(Duration::from_secs(60));
    assert_eq!(budget.remaining(), Duration::from_secs(120));
}
