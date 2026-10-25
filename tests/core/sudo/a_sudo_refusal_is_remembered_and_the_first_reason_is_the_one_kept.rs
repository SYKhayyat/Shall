use super::*;

/// A refusal is remembered, and the first reason is the one kept.
///
/// **The mechanism is what is asserted, not sudo.** A test that needs a real password prompt
/// on a real tty is the container harness's job (`[16f]`), and one that shells out to `sudo`
/// here would answer differently on every developer's machine.
///
/// It lives here rather than in `tests/core/sudo/` because it reads the memo's own cells: a
/// test crate links the library without `cfg(test)`, so it cannot.
#[test]
fn a_sudo_refusal_is_remembered_and_the_first_reason_is_the_one_kept() {
    let state = SudoState::new();
    assert!(
        state.refusal().is_none(),
        "a fresh run starts with no refusal recorded, so nothing below means anything"
    );

    let first = state.refuse("no terminal to ask on".to_string());
    assert!(first.to_string().contains("no terminal"));
    // A later caller's guess must not overwrite the reason the first one established: the
    // *first* refusal is the one that happened, and everything after it is this memo.
    let _ = state.refuse("something else entirely".to_string());
    assert_eq!(state.refusal().as_deref(), Some("no terminal to ask on"));
}
