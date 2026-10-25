use super::*;

/// `Work = (Work | editors)` used to resolve Work for ever: set math re-enters the
/// resolver one level below the trail the profile loader keeps, so every pass saw a
/// fresh one, and the run died with a stack overflow (exit 0xC00000FD) and no
/// diagnostic. The expression trail is what stands between that line and the overflow.
#[test]
fn a_profile_whose_set_math_refers_to_itself_is_an_error_not_an_overflow() {
    let f = fx(
        "Work\n",
        &[("Work", "(Work | editors)\n")],
        &[("editors.txt", "apt:vim\n")],
    );
    let err = resolve(&f).expect_err("a self-referential set expression must refuse");
    let msg = err.to_string();
    assert!(
        msg.contains("set math"),
        "the error must name the door it came through: {msg}"
    );
    assert!(msg.contains("Work"), "the error must name the loop: {msg}");
}
