use super::*;

/// Three managers on a bare runner cannot overlap 2×, and a gate that says they should is a
/// gate people learn to ignore (`lifecycle-floor.txt` says the same about guessed constants).
#[test]
fn too_few_children_is_not_a_measurement() {
    let shape = Class::of("list").shape().unwrap();
    assert!(shape_violation(
        shape,
        2,
        Duration::from_millis(2_000),
        Duration::from_millis(2_000),
        Duration::from_millis(2_000),
        2
    )
    .is_none());
}
