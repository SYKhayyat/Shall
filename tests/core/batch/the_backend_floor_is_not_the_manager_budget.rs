use super::*;

/// **A backend's floor is the larger of two budgets, and never more than the run.**
///
/// The manager default is five minutes, which is a generous budget for `apt install` and not
/// enough for a command that rebuilds a whole system. `#225` is what the difference cost: the
/// rebuild was cancelled at the manager budget and everything downstream read the cancellation
/// as a finished command.
#[test]
fn a_backend_floor_raises_the_budget_it_cannot_fit_in() {
    assert_eq!(
        deadline_for(NODE, 1, TOTAL, None),
        NODE,
        "a backend with no floor gets exactly the manager arithmetic"
    );
    assert_eq!(
        deadline_for(NODE, 1, TOTAL, Some(Duration::from_secs(1800))),
        Duration::from_secs(1800),
        "a floor above the manager budget is the budget"
    );
}

/// A floor below the arithmetic changes nothing — the floor is a minimum, not a cap.
#[test]
fn a_floor_under_the_arithmetic_is_ignored() {
    assert_eq!(
        deadline_for(NODE, 4, TOTAL, Some(Duration::from_secs(60))),
        NODE * 4
    );
}

/// **A floor above the transaction could never be observed**, so it is capped by the run's total
/// for the same reason `node_deadline` caps the batch — a bound nobody can reach is a sentence
/// about a decision that was never made.
#[test]
fn a_floor_cannot_outlive_the_transaction_it_sits_in() {
    assert_eq!(
        deadline_for(NODE, 1, TOTAL, Some(Duration::from_secs(7200))),
        TOTAL
    );
}
