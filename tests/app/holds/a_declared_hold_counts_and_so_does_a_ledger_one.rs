use super::*;

#[test]
fn a_declared_hold_counts_and_so_does_a_ledger_one() {
    let mut state = StateRegistry::default();
    state.hold("cargo:ripgrep");
    let d = desired(vec![
        spec("npm", "typescript", true),
        spec("npm", "eslint", false),
    ]);
    let holds = Holds::new(&state, Some(&d), None);

    assert!(holds.contains("cargo", "ripgrep"), "the ledger entry");
    assert!(holds.contains("npm", "typescript"), "the declared one");
    assert!(!holds.contains("npm", "eslint"), "no `@hold` on this line");
    assert_eq!(holds.len(), 2);
}
