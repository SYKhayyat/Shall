use super::*;

/// The test process has no terminal on stdin, so these exercise the third answer directly.
#[test]
fn a_refusing_prompt_with_nobody_there_refuses_by_name() {
    let e = confirm_in(
        false,
        false,
        "Remove these packages?",
        Unattended::Refuse("Refusing to remove without confirmation. Re-run with --yes."),
    )
    .expect_err("no terminal, and this site refuses");
    assert!(
        matches!(e, Error::Refused(_)),
        "a prompt nobody could answer must be a refusal, not an IO error: {e:?}"
    );
    assert!(e.to_string().contains("--yes"), "{e}");
}
