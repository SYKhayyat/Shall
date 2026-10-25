use super::*;

/// The two are released by different commands, and the answer has to say which.
#[test]
fn each_hold_is_released_by_the_command_that_made_it() {
    let mut state = StateRegistry::default();
    state.hold("cargo:ripgrep");
    let d = desired(vec![spec("npm", "typescript", true)]);
    let holds = Holds::new(&state, Some(&d), None);

    assert_eq!(holds.release("cargo", "ripgrep"), "`shall unhold`");
    assert_eq!(
        holds.release("npm", "typescript"),
        "remove `@hold` from the line"
    );
}
