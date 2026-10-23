use super::*;

/// G-5. One `scoop` typo printed ~110 lines of unrelated bucket commits at the user, with
/// raw SGR sequences, and the sentence that mattered was the fourth of them. The stream is
/// reproduced here in its real shape: a coloured banner, a long body, the complaint early.
#[test]
fn a_failed_command_shows_the_line_that_explains_it_and_not_the_bucket_update() {
    let mut stream =
        String::from("\u{1b}[32mUpdating Scoop...\u{1b}[0m\nUpdating 'main' bucket...\n");
    stream.push_str("Couldn't find manifest for 'definitely-not-real-xyz123'.\n");
    for i in 0..110 {
        stream.push_str(&format!("   \u{1b}[33m* commit {i} in main\u{1b}[0m\n"));
    }
    let err = executor_for(exit_policy::scoop())
        .ensure_status("scoop", finished(0, &stream, ""))
        .unwrap_err();
    let msg = err.to_string();

    assert!(
        msg.contains("Couldn't find manifest for 'definitely-not-real-xyz123'"),
        "the message dropped the one line that explains the failure:\n{msg}"
    );
    assert!(
        !msg.contains('\u{1b}'),
        "an escape sequence reached the user's terminal; the machine was protected from \
         these and the human was not:\n{msg:?}"
    );
    assert!(
        !msg.contains("commit 42 in main"),
        "the whole bucket update was pasted at the user:\n{msg}"
    );
    assert!(
        msg.lines().count() <= 10,
        "{} lines of a manager's output reached the user; a failure names one place to \
         look:\n{msg}",
        msg.lines().count()
    );
    assert!(
        msg.contains("more line(s) of output"),
        "output was dropped and not accounted for; a count with nowhere to look is not one \
         place to look:\n{msg}"
    );
}
