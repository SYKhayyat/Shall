use super::*;

/// Nothing declared is `any == false`, which is what keeps Shall out of `/etc/nixos` on a
/// machine that never asked it to write there.
#[test]
fn a_state_with_neither_declares_nothing_and_says_so() {
    let d = declared(&state(vec![])).expect("an empty state");
    assert_eq!(d, Declared::default());
    assert!(!d.any);
}
