use super::*;

/// **And the cap did not swallow the shape.** A fix that flattened everything to the total
/// timeout would pass the test above and give one package an hour.
#[test]
fn the_batch_still_scales_with_its_size_below_the_ceiling() {
    assert_eq!(node_deadline(NODE, 2, TOTAL), Duration::from_secs(600));
    assert_eq!(node_deadline(NODE, 4, TOTAL), Duration::from_secs(1200));
    assert_eq!(node_deadline(NODE, 8, TOTAL), Duration::from_secs(2400));
}
