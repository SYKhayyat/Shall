use super::*;

/// The documented precedence that must survive the change above: a manager printing both a
/// permanent and a transient marker fails fast rather than looping.
#[test]
fn a_permanent_marker_still_outranks_a_transient_one() {
    let p = cargo();
    assert_eq!(
        p.retryability(&ExitPolicy::haystack(
            b"spurious network error\nerror: there are no binaries",
            b""
        )),
        Retryability::Permanent
    );
}
