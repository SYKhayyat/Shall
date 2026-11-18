//! A range that cannot be split must be said so, not split into itself.
//!
//! **The narrowing is a worklist of `(lo, hi)` and it terminates because every push is smaller
//! than its parent.** Nothing else enforces that: `mid = lo + (hi - lo) / 2` is right, and the
//! halves are smaller, so the loop empties. Change either operator and the argument collapses —
//! `mid = lo * (hi - lo) / 2` is `0` for the very first range, whose halves are `(0, 0)` and
//! `(0, n)`: the parent again, forever.
//!
//! **That is not a hypothetical.** The night of 2026-11-17 lost the shard holding exactly those
//! two mutants (`+` → `*` and `/` → `%` at the split) three runs in a row, at eleven minutes each,
//! while every other shard answered: the narrowing issued manager commands without end, each
//! answer grew the run's memory, and the runner was killed for starvation before any test timeout
//! could name a mutant (#190). A gate that dies is a gate that reports nothing.
//!
//! So the split point answers `None` when there is nothing to split, the caller has to handle it,
//! and this file pins the property that makes the worklist empty: **inside, or nothing.**

use shall::core::batch::midpoint;

#[test]
fn a_range_of_two_or_more_has_a_split_point_strictly_inside_it() {
    for lo in 0..=64usize {
        for hi in lo..=64usize {
            match midpoint(lo, hi) {
                Some(mid) => assert!(
                    mid > lo && mid < hi,
                    "midpoint({lo}, {hi}) = {mid}, which is not strictly inside the range: the \
                     halves would include the parent and the narrowing would never finish"
                ),
                None => assert!(
                    hi - lo < 2,
                    "midpoint({lo}, {hi}) said there is nothing to split, but the range holds \
                     {} members",
                    hi - lo
                ),
            }
        }
    }
}

#[test]
fn a_single_member_is_never_reported_as_splittable() {
    for lo in 0..=64usize {
        assert_eq!(midpoint(lo, lo), None, "an empty range has no split point");
        assert_eq!(
            midpoint(lo, lo + 1),
            None,
            "one member is not two halves, however it is cut"
        );
    }
}

#[test]
fn cutting_at_the_split_point_reaches_singletons_in_a_bounded_number_of_cuts() {
    // The worklist narrows; this is the arithmetic behind "the worklist empties". A bisection of
    // `n` members takes at most `n - 1` successful cuts, and each cut visits both halves, so the
    // whole search is bounded by `2 * n` pops - which is also the guard's budget in
    // `narrow_batch`. If a cut ever failed to shrink, this loop would not return.
    for n in 2..=257usize {
        let mut work = vec![(0usize, n)];
        let mut cuts = 0usize;
        let mut pops = 0usize;
        while let Some((lo, hi)) = work.pop() {
            pops += 1;
            assert!(pops <= 2 * n + 2, "n = {n}: the worklist is not emptying");
            let Some(mid) = midpoint(lo, hi) else {
                assert_eq!(hi - lo, 1, "n = {n}: {lo}..{hi} was refused a split point");
                continue;
            };
            cuts += 1;
            work.push((lo, mid));
            work.push((mid, hi));
        }
        assert_eq!(
            cuts,
            n - 1,
            "n = {n}: a bisection takes one cut per member but the last"
        );
    }
}
