use super::*;

/// Trojan source, in a package manager's output rather than in a module: U+202E reverses
/// everything after it as it renders, so a failure can be made to read as its opposite.
/// The grammar's refusals have named it by codepoint since B38; a command's output did not.
#[test]
fn an_invisible_character_in_a_managers_output_is_named_not_drawn() {
    let err = executor_for(ExitPolicy::default())
        .ensure_status("somepm", finished(1, "", "failed: \u{202E}drowssap\n"))
        .unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("<U+202E>"),
        "the override was not named:\n{msg}"
    );
    assert!(
        !msg.contains('\u{202E}'),
        "the override was reprinted at the terminal"
    );
}
