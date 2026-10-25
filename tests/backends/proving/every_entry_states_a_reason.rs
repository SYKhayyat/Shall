use super::*;

/// Every reason says something. A blank excuse is the exemption this table exists to stop
/// being possible — `Q17`'s bar is that a harness *cannot*, and a reason nobody wrote is
/// indistinguishable from a reason nobody has.
#[test]
fn every_entry_states_a_reason() {
    for (backend, why) in UNPROVEN {
        assert!(
            why.len() > 40,
            "`{backend}` is exempted with {} characters of reason; say why a harness \
             cannot reach it",
            why.len()
        );
        assert!(!backend.is_empty());
    }
}
