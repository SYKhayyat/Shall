use super::*;

/// The reported bug: a typo behind a real backend prefix. `scoop` resolves, so this is
/// never `Unresolvable`; scoop's own policy says its output means the name is not there,
/// and until that was read the line stayed in `modules/imperative.txt` and wedged every
/// later command.
#[test]
fn a_failure_that_says_the_name_is_absent_is_withdrawn() {
    let e = boxed(Error::command_failed_absent(
        "`scoop` failed: Couldn't find manifest for 'definitely-not-real'.",
    ));
    assert!(says_a_name_is_absent(&e));
    assert_eq!(
        absent_command_message(&e),
        Some("`scoop` failed: Couldn't find manifest for 'definitely-not-real'.")
    );
}
