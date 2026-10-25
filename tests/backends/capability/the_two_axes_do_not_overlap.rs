use super::*;

#[test]
fn the_two_axes_do_not_overlap() {
    for b in SELECTS_ARTIFACTS {
        assert!(
            !has_channels(b),
            "{} would accept both formats and channel",
            b
        );
    }
}
