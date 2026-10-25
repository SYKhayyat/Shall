use super::*;

#[test]
fn a_missing_identity_is_named_as_the_reason_the_commit_failed() {
    // Reachable only since Shall stopped injecting `shall@localhost`: git's own message is
    // about `user.email`, and the hint has to be about how Shall uses it.
    let hint = commit_refusal_hint(
        "*** Please tell me who you are.

fatal: unable to auto-detect email address",
    );
    assert!(hint.contains("git config --global user.email"), "{}", hint);
    let signing = commit_refusal_hint("error: gpg failed to sign the data");
    assert!(signing.contains("commit.gpgsign"), "{}", signing);
}
