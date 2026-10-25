use super::*;

/// Terminal members are recorded verbatim; whether each is a real backend is the grammar's
/// check, not this one. A group is a shortcut for a chain, and the chain's parts are
/// validated where every other prefix's parts are.
#[test]
fn members_are_recorded_not_validated_here() {
    let g = Groups::parse("x = notabackend\n").unwrap();
    assert_eq!(g.expand("x"), Some(["notabackend".to_string()].as_slice()));
}
