use super::*;

/// **The cap, which is the whole of `PLAN.md` #82's buildable half.** `total_timeout` wraps
/// the whole transaction, so a node budget above it names a deadline the run cannot reach:
/// `300s × 16` is eighty minutes inside a sixty-minute run. Capping changes no outcome — the
/// transaction ends first either way — and makes the two constants' relationship a fact.
#[test]
fn no_batch_is_given_a_budget_the_transaction_would_have_ended_first() {
    for n in [1usize, 8, 12, 16, 17, 100, 1000] {
        let got = node_deadline(NODE, n, TOTAL);
        assert!(
            got <= TOTAL,
            "a batch of {n} was given {got:?} inside a {TOTAL:?} transaction"
        );
    }
    // The shape it had before, said out loud so the number cannot quietly come back.
    assert_eq!(
        NODE.saturating_mul(16),
        Duration::from_secs(4800),
        "sixteen packages at five minutes each is eighty minutes"
    );
    assert_eq!(node_deadline(NODE, 16, TOTAL), TOTAL);
}
