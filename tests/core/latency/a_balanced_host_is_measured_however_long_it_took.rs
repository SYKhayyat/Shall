use super::*;

/// The escape is for skew, and **not** for a run that is merely slow — a run whose slowest
/// child is a small share of the summed time can always reach the floor, however long it took.
#[test]
fn a_balanced_host_is_measured_however_long_it_took() {
    let shape = Class::of("list").shape().unwrap();
    // Five children, none holding more than a fifth of the time: ceiling is ~5x.
    assert!(
        shape_measurement_gap(
            shape,
            5,
            Duration::from_millis(5_000),
            Duration::from_millis(1_000),
            1,
        )
        .is_none(),
        "a balanced fan-out has a measurement, and this host is judged by the floor"
    );
    // And a slow one is judged, not excused: 4s of the slowest against a 1.25x floor over
    // 5s summed still overlaps 1.25x exactly, which is the boundary.
    let at_the_floor = shape_violation(
        shape,
        5,
        Duration::from_millis(5_000),
        Duration::from_millis(1_250),
        Duration::from_millis(4_000),
        1,
    );
    assert!(
        at_the_floor.is_none(),
        "exactly at the floor is inside it, and it was measured to get there: {at_the_floor:?}"
    );
}
