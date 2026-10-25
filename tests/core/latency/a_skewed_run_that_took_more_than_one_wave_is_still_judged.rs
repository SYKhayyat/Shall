use super::*;

/// **A run that is skewed *and* took more than one wave is judged on both halves.** The
/// exemption is for "the ratio cannot see this host", not for "the ratio is inconvenient".
#[test]
fn a_skewed_run_that_took_more_than_one_wave_is_still_judged() {
    let shape = Class::of("list").shape().unwrap();
    let summed = Duration::from_millis(460);
    let slowest = Duration::from_millis(360);
    assert!(
        shape_measurement_gap(shape, 5, summed, slowest, 2).is_none(),
        "more than one wave is not evidence of overlap, so the ratio is read as usual"
    );
    let reported = shape_violation(shape, 5, summed, slowest, Duration::from_millis(460), 2)
        .expect("1.0x under a 1.25x floor is a fault, whatever the children look like");
    assert!(reported.contains("overlap"), "{reported}");
}
