use super::*;

/// The regression, end to end and in its own numbers.
///
/// macOS nightly, 2026-08-14: 43 packages in the state file, 276 undeclared found through
/// the managers `priority` names. Scoped, the management side is what those same managers
/// account for — and the refusal comes back.
#[test]
fn the_macos_nightly_refuses_once_both_sides_count_the_same_machine() {
    // The unscoped count is what shipped, and it cleared the bar.
    assert!(
        !reads_as_a_mistake(43, 276),
        "43/276 is 0.156 — this is the number that let the purge through"
    );
    // Scoped to the managers that answered, the same machine reads as a mistake. Any
    // numerator below 27.6 does; the point is that dropping the managers which cannot
    // appear on the deletion side can only move it down.
    assert!(reads_as_a_mistake(12, 276));
}
