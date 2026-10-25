use super::*;

/// **An empty `declared` set means two different things and the type now says which.**
///
/// `shall hold` with an unresolvable manifest wrote a correct warning to stderr and then
/// printed `No packages are held.` to stdout at exit 0 — so the one consumer that matters,
/// a script reading the answer, got a confident falsehood (B3). `upgrade` keeping the same
/// tolerance is right, which is why this is a fact the caller reads rather than a failure
/// the constructor raises.
#[test]
fn a_manifest_that_would_not_resolve_is_not_a_manifest_with_no_holds() {
    let state = StateRegistry::default();

    let answered = Holds::new(&state, Some(&desired(vec![])), None);
    assert!(answered.is_empty());
    assert_eq!(
        answered.unresolved(),
        None,
        "a model that resolved and held nothing is an answer"
    );

    let could_not_look = Holds::new(&state, None, Some("modules/dev.txt:3: bad line".into()));
    assert!(could_not_look.is_empty());
    assert_eq!(
        could_not_look.unresolved(),
        Some("modules/dev.txt:3: bad line"),
        "the two produce an identical empty list and only one of them is a clean bill"
    );
}
