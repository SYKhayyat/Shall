use super::*;

/// Junk parses to nothing rather than to a number that would silently resize a volume.
#[test]
fn what_is_not_a_size_is_not_guessed_at() {
    for junk in [
        "", "  ", "big", "10X", "G10", "-5G", "10 G B", "10Gigs", "10.5.5G",
    ] {
        assert_eq!(parse_size(junk), None, "{} parsed as a size", junk);
    }
}
