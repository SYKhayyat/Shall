use super::*;

/// A crawl that answered for nothing cannot license a deletion.
///
/// Zero managed over any deletion list is a ratio of zero, which is below any threshold —
/// asserted rather than assumed, because `0.0 / n` is the one input where a float
/// comparison could plausibly have been written the other way round.
#[test]
fn managing_nothing_is_always_a_mistake() {
    assert!(reads_as_a_mistake(0, 1));
    assert!(reads_as_a_mistake(0, 576));
    assert_eq!(managed_where_the_crawl_could_see([].iter(), &[]), 0);
}
