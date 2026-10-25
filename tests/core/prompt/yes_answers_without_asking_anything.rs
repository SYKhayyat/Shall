use super::*;

/// `--yes` answers before anything else is consulted, which is what makes it usable from a
/// script: it must not depend on whether a terminal happens to be attached.
#[test]
fn yes_answers_without_asking_anything() {
    assert!(confirm(true, "?", Unattended::Refuse("never reached")).expect("--yes"));
    assert!(confirm(true, "?", Unattended::Decline("never printed")).expect("--yes"));
}
