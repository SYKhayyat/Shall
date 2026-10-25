use super::*;

/// The other right answer: an offer declines and the run continues. Returning an error here
/// would turn "no package manager to install with" into a failed sync.
#[test]
fn a_declining_prompt_with_nobody_there_answers_no() {
    let answered = confirm_in(
        false,
        false,
        "Run that to install brew?",
        Unattended::Decline("Not asking in a non-interactive shell."),
    )
    .expect("declining is not an error");
    assert!(!answered);
}
