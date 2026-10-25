use super::*;

/// **The bound cannot be passed by an archive that declares more bytes than memory.**
///
/// Every declared size in an archive is a field its author wrote, so the arithmetic that
/// adds them is inside the trust boundary — and `Iterator::sum` for integers **wraps in a
/// release build**, so a zip declaring `u64::MAX` and `2` totalled `1` and sat under any
/// `max_unpacked_bytes` (`PLAN.md` #86). The tar side already walked with `saturating_add`;
/// this is the same rule under a name both sides call.
#[test]
fn the_unpacked_total_saturates_rather_than_wrapping() {
    // The release-build answer, spelled out rather than reached through `sum`, because
    // `sum` PANICS on overflow in a debug build and this test runs in one. What Shall ships
    // is a release build, and there the same expression is 1 — asserted here so the case
    // below is known to be about a real hazard rather than about arithmetic that cannot
    // happen.
    assert_eq!(
        [u64::MAX, 2u64]
            .iter()
            .copied()
            .fold(0u64, u64::wrapping_add),
        1
    );

    // The rule, on the numbers a crafted archive carries: a total past the address space is
    // the largest total there is, and it is over every cap anybody can configure.
    assert_eq!(declared_unpacked_total([u64::MAX, 2]), u64::MAX);
    assert!(declared_unpacked_total([u64::MAX, 2]) > 1_000_000_000);
    // The incremental form the tar branch uses, on the same pair.
    assert_eq!(add_unpacked(add_unpacked(0, u64::MAX), 2), u64::MAX);

    // And the ordinary cases are untouched, because a fix that changes real totals is a
    // different fix: a real archive's total is its total.
    assert_eq!(declared_unpacked_total([1u64, 2, 3]), 6);
    assert_eq!(declared_unpacked_total(std::iter::empty::<u64>()), 0);
    assert_eq!(add_unpacked(0, 0), 0);
}
