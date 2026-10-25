use super::*;

/// **A kind with no sub-category refuses a qualifier rather than matching nothing**, and
/// the refusal points at the thing that does work: naming the item.
#[test]
fn a_flat_kind_refuses_a_qualifier_and_says_what_to_type_instead() {
    for kind in ALL.iter().filter(|k| k.subdivision().is_none()) {
        let word = format!("{}:something", kind.as_str());
        let err = LockSelection::parse(&word, &[]).expect_err(&word);
        let msg = err.to_string();
        assert!(msg.contains(kind.as_str()), "{msg}");
        assert!(
            msg.contains("its own name"),
            "the refusal must point at what does work: {msg}"
        );
    }
    // And the four that do divide accept one, which is the other half of the same claim.
    for kind in ALL.iter().filter(|k| k.subdivision().is_some()) {
        let word = format!("{}:something", kind.as_str());
        assert!(
            LockSelection::parse(&word, &[]).is_ok(),
            "{kind} divides and must accept a qualifier"
        );
    }
}
