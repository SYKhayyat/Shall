use super::*;

/// An error prefix is a line's opening word, not a substring of prose. A package whose
/// description mentions errors is not a failed command.
#[test]
fn a_prefix_marker_anchors_to_a_line_and_does_not_match_prose() {
    let p = nimble();
    assert!(!p.signals_failure(&ExitPolicy::haystack(
        b"jsony - a library with no error: handling at all",
        b""
    )));
    assert!(p.signals_failure(&ExitPolicy::haystack(
        b"building...\n    Error:  Package not found\n",
        b""
    )));
}
