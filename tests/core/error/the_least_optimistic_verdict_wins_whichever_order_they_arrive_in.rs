use super::*;

/// **The rule is a total order, and every pair must obey it.** A summary that took the
/// *first* verdict, or the *last*, reads the same on a one-failure run and disagrees with
/// itself the moment two failures arrive in a different order.
#[test]
fn the_least_optimistic_verdict_wins_whichever_order_they_arrive_in() {
    use Retryability::*;
    // Least to most dominant. Every later entry must beat every earlier one, both ways
    // round, and each must be a fixed point against itself.
    let ranked = [Transient, Exhausted, Unknown, Permanent];
    for (i, &lo) in ranked.iter().enumerate() {
        assert_eq!(lo.and_also(lo), lo, "{lo:?} disagreed with itself");
        for &hi in &ranked[i + 1..] {
            assert_eq!(lo.and_also(hi), hi, "{lo:?}.and_also({hi:?})");
            assert_eq!(hi.and_also(lo), hi, "{hi:?}.and_also({lo:?})");
        }
    }
}
