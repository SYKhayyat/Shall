use super::*;

/// A rule Shall cannot express must be refused, not approximated: a firewall line that
/// half-applies is a perimeter nobody can reason about.
#[test]
fn anything_else_is_refused_with_a_reason() {
    for bad in [
        "22",
        "22/sctp",
        "http/tcp",
        "0/tcp",
        "99999/tcp",
        "default/sideways",
    ] {
        let err = Rule::parse(bad).unwrap_err();
        assert!(!err.is_empty(), "{} produced an empty error", bad);
    }
}
