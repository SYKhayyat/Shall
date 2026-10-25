use super::*;

#[test]
fn the_table_answers_both_ways() {
    assert!(is_proven("apt"), "apt has a lifecycle in every Linux image");
    assert!(!is_proven("mas"), "mas needs hardware no runner here has");
    assert_eq!(unproven_reason("mas"), Some(NO_HARNESS_RAN_IT));
    assert_eq!(unproven_reason("apt"), None);
}
