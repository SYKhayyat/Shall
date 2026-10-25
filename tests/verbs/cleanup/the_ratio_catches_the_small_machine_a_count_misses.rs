use super::*;

#[test]
fn the_ratio_catches_the_small_machine_a_count_misses() {
    // Alpine: adopt correctly took 14 packages, and a mis-scoped removal scheduled all
    // 14 — under any count limit, none protected, all things you would cry about.
    assert!(reads_as_a_mistake(1, 14));
    // And an adopted Alpine is fine: 14 managed, a handful of strays to clear.
    assert!(!reads_as_a_mistake(14, 20));
}
