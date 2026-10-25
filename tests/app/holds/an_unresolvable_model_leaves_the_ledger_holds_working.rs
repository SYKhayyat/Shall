use super::*;

/// A model that will not resolve yields the ledger alone rather than failing: `upgrade` is
/// not a command that needs the manifest to parse.
#[test]
fn an_unresolvable_model_leaves_the_ledger_holds_working() {
    let mut state = StateRegistry::default();
    state.hold("cargo:ripgrep");
    let holds = Holds::new(&state, None, None);
    assert!(holds.contains("cargo", "ripgrep"));
    assert_eq!(holds.len(), 1);
}
