use super::*;

#[test]
fn a_chain_only_accepts_what_it_lists() {
    let priority: Vec<String> = ["apt", "snap"].iter().map(|s| s.to_string()).collect();
    let chain = Candidates::Named(vec!["apt".into()]);
    assert!(chain.accepts("apt", &priority));
    // The lock naming `snap` is not an answer to this line, even though the host lists it.
    assert!(!chain.accepts("snap", &priority));
    assert!(Candidates::Priority.accepts("snap", &priority));
}
