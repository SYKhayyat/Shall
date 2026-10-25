use super::*;

/// `--yes` answers before anything is consulted, terminal or not.
#[test]
fn an_attended_confirm_still_lets_yes_answer_first() {
    assert!(confirm_in(true, true, "?", Unattended::Refuse("never reached")).expect("--yes"));
    assert!(
        confirm_in(true, true, "?", Unattended::Decline("never printed")).expect("--yes"),
        "--yes must win on the attended path too"
    );
}
