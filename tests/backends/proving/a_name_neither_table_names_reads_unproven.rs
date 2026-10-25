use super::*;

/// The polarity, in the direction that used to be wrong. A name neither table names is a
/// backend nobody has written anything about, and it reads unproven — which is the flip, and
/// the only part of the flip a caller can see.
#[test]
fn a_name_neither_table_names_reads_unproven() {
    assert_eq!(
        unproven_reason("a_backend_added_tomorrow"),
        Some(NO_HARNESS_RAN_IT)
    );
    assert!(!is_proven("a_backend_added_tomorrow"));
}
