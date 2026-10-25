use super::*;

/// **A selection that comes out empty is refused.** Every route to it, because the empty
/// set is the one outcome indistinguishable from success.
#[test]
fn a_selection_that_comes_out_empty_is_an_error_rather_than_a_quiet_no_op() {
    for (what, except) in [
        ("everything", words(&["everything"])),
        ("exec", words(&["exec"])),
        ("packages", words(&["versions", "backends"])),
        ("packages", words(&["packages"])),
        ("", vec![]),
    ] {
        assert!(
            LockSelection::parse(what, &except).is_err(),
            "an empty selection must be refused: {what:?} minus {except:?}"
        );
    }
}
