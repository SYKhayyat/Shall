use super::*;

/// The shape budget catches the regression the second budget cannot see.
///
/// A change that serialises the fan-out leaves the wall clock inside a ceiling of `None`
/// for ever. These are the same numbers measured on Windows with 24 ready backends —
/// 23 children, 23.67 s of child time in 3.75 s of wall clock — against the same run with
/// the overlap taken out of it.
#[test]
fn a_serialised_fan_out_is_out_of_budget_and_a_real_one_is_not() {
    let shape = Class::of("list")
        .shape()
        .expect("`list` asks every manager");

    // Real readings from three hosts, including the one that failed the first draft's
    // numbers: 2.0x against a floor of 2.0, and three waves against a ceiling of two, on a
    // runner doing nothing wrong.
    for (children, summed_ms, wall_ms, waves, host) in [
        (23, 23_670, 3_750, 2, "Windows, release: 6.3x"),
        (23, 32_620, 5_730, 2, "Windows, debug: 5.7x"),
        (
            16,
            8_440,
            4_270,
            3,
            "ubuntu-latest, CI run 31517073405: 2.0x",
        ),
        (9, 3_340, 1_760, 3, "macos-latest, CI run 31517073405: 1.9x"),
    ] {
        // The slowest child is read off the same `--timings` lines: a run whose wall clock
        // is under `summed / floor` cannot have been this overlapped, so the wall clock is
        // itself a lower bound on the slowest child and is what these hosts were checked at.
        // A host whose slowest child really was longer is caught by `shape_measurement_gap`
        // and skipped, which is the behaviour the next test pins.
        let slowest_ms = wall_ms;
        let healthy = shape_violation(
            shape,
            children,
            Duration::from_millis(summed_ms),
            Duration::from_millis(slowest_ms),
            Duration::from_millis(wall_ms),
            waves,
        );
        assert!(
            healthy.is_none(),
            "a measured healthy run is reported as a violation ({host}): {healthy:?}"
        );
    }

    // The same children, run one after another. Wall clock == summed, so overlap is 1.0.
    let serial = shape_violation(
        shape,
        23,
        Duration::from_millis(23_670),
        // One at a time, so the slowest child *is* the whole wall clock — the ceiling is
        // 1.0 and a serial run is therefore judged rather than skipped, which is the case
        // that must not be given the skew escape.
        Duration::from_millis(23_670),
        Duration::from_millis(23_670),
        23,
    );
    let serial = serial.expect(
        "a fan-out that overlapped nothing is inside budget, which is the regression \
         `Class::EveryBackend => None` could not see",
    );
    assert!(serial.contains("overlap"), "{serial}");
    assert!(serial.contains("wave"), "{serial}");
}
