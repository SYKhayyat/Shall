use shall::core::exit::Exit;

/// A guard refusal must be distinguishable from a crash and from a difference — that is
/// the separation the whole table exists for.
#[test]
fn a_refusal_is_neither_a_failure_nor_a_difference() {
    assert_ne!(Exit::Refused.code(), Exit::Failed.code());
    assert_ne!(Exit::Refused.code(), Exit::Differences.code());
}
