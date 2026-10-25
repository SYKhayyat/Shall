use super::*;

#[test]
fn a_binary_this_table_has_never_heard_of_does_not_terminate() {
    // The default the header states, asserted rather than remembered: an unlisted manager
    // is one nobody checked, and the safe reading of "nobody checked" is "no terminator".
    assert!(!terminates_options("some-manager-invented-tomorrow"));
}
