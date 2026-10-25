use super::*;

/// **A malformed `--temp` is refused, and refusing it is not the same as dying.**
///
/// The unit is read as a `char` and the number as a byte range. Taking one byte off the
/// end landed inside a multi-byte character, and with `panic = "abort"` the process was
/// gone before `suspend` could turn the `None` into the message it is written to produce.
/// The oversized cases cover the other half: the multiplication used to wrap, which turned
/// a far-future restore time into one already past.
#[test]
fn a_malformed_duration_is_refused_rather_than_fatal() {
    for bad in [
        "7\u{434}",  // Cyrillic unit, two bytes
        "7\u{4e00}", // three bytes
        "\u{434}",   // no digits at all
        "7x",
        "7",
        "",
        "-1d",
        "99999999999999999d",    // overflows u64 seconds
        "18446744073709551615s", // overflows `now + seconds`
    ] {
        assert_eq!(
            StateRegistry::parse_duration(bad),
            None,
            "`{}` must be refused",
            bad
        );
    }
}
