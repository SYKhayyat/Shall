use super::*;

/// A bare ledger entry matches under any backend — that is what the ledger's own matching
/// does, and this union must not narrow it.
#[test]
fn a_bare_ledger_entry_still_matches_any_backend() {
    let mut state = StateRegistry::default();
    state.hold("curl");
    let holds = Holds::new(&state, None, None);
    assert!(holds.contains("apt", "curl"));
    assert!(holds.contains("brew", "curl"));
    assert!(!holds.contains("apt", "wget"));
}
