use super::*;

/// **The case this whole change exists for: a perfect fan-out on a skewed host.**
///
/// Five managers, one of which holds most of the summed time — the five readings in
/// `PLAN.md` #90, where `emacs --batch` alone took 0.36s of 0.46s. Every run starts all five
/// children at the same instant and finishes in one wave, and the ratio cannot exceed
/// `0.46 / 0.36 = 1.28x` against a floor of 1.25x. A scheduler that serialised the lot would
/// score 1.0x; a scheduler that overlapped everything scores 1.28x. There is no reading in
/// between, so the floor is not measuring this host.
///
/// Asserted in both directions, because the two halves fail differently and only one of them
/// is a false failure:
///
/// - the skewed perfect run must not be reported as a violation — the false failure #90 filed;
/// - **and a serial run over the same skewed children must still be caught** — by the wave
///   ceiling, which is the half of the pair that does not depend on child durations. That is
///   what a lowered floor would have thrown away, and why the fix is a skip and not a number.
#[test]
fn a_skewed_host_cannot_measure_the_ratio_but_can_still_measure_collapse() {
    let shape = Class::of("list")
        .shape()
        .expect("`list` asks every manager");
    // The numbers off the failing host, to the hundredth of a second they were reported in.
    let summed = Duration::from_millis(460);
    let slowest = Duration::from_millis(360);
    let wall = Duration::from_millis(420); // 460/420 = 1.1x, the reported reading.
    assert!(
        overlap_ceiling(summed, slowest) < shape.min_overlap * shape.overlap_headroom,
        "the ceiling must be below the reachable floor for this test to mean anything: \
         ceiling {:.2}x, reachable {:.2}x",
        overlap_ceiling(summed, slowest),
        shape.min_overlap * shape.overlap_headroom
    );

    // One wave, which is the signature of a perfect fan-out.
    assert!(
        shape_violation(shape, 5, summed, slowest, wall, 1).is_none(),
        "a one-wave run on a skewed host is a perfect fan-out, not a violation"
    );

    // The skip says so out loud, and names the number that could not be reached.
    let gap = shape_measurement_gap(shape, 5, summed, slowest, 1).expect(
        "this host has no \
                 measurement",
    );
    assert!(gap.contains("no scheduler here can score"), "{gap}");
    assert!(gap.contains("one wave"), "{gap}");

    // **The control, and the reason the gap needs the `waves` condition at all.** The same
    // children, serialised: five waves, so `slowest == summed` and the ceiling is 1.0 — a
    // serial run looks maximally skewed *by construction*. Without requiring evidence of
    // concurrency, the very run this gate exists to catch would claim the exemption and go
    // unreported, which is what the first draft of this fix did; this assertion is what
    // caught it.
    let serial = shape_violation(shape, 5, summed, slowest, summed, 5)
        .expect("a serial fan-out is a serial fan-out on a skewed host too");
    assert!(serial.contains("5 wave(s)"), "{serial}");
    assert!(
        serial.contains("overlap"),
        "and the overlap half judges it too, because a run of more than one wave has not \
         demonstrated that it overlapped anything: {serial}"
    );
}
