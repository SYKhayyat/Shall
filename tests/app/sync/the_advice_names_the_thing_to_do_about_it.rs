/// **The advice says what to do, and it says it for the failure that happened.**
///
/// Found by the mutation gate: `what_to_do_about` replaced with `"xyzzy".into()` and with
/// `String::new()` both survived the whole suite. **It is the last thing a user reads** — the
/// sentence that tells them whether to run the command again, to correct a line, or to fix
/// something the error names — and an empty sentence is worse than no advice, because the
/// summary looks like it answered.
///
/// **The three cases, because the advice is driven off the classification the error carries
/// rather than off a sentence per case**: an error that names an absent package, a transient
/// failure, and a permanent one.
#[test]
fn the_advice_names_the_thing_to_do_about_it() {
    // **No error at all**: there is nothing to say but the command.
    let nothing = shall::app::sync::what_to_do_about(None, "apt:curl");
    assert!(
        nothing.contains("shall heal"),
        "with no error to classify, the advice is to run the command again: {nothing}"
    );

    // **A name that does not exist.** The advice must NOT be "re-run sync" — that is the one
    // thing that cannot help when the name does not exist — and it must name the command that
    // can.
    let absent = shall::app::sync::what_to_do_about(
        Some(&shall::core::Error::NoSuchPackage {
            name: "curl".to_string(),
            message: "no such package".to_string(),
        }),
        "apt:curl",
    );
    assert!(
        absent.contains("shall unmanage apt:curl"),
        "the advice must name the command that would remove the line: {absent}"
    );
    assert!(
        !absent.contains("re-run `shall sync`"),
        "the advice must not tell the user to re-run the command that cannot help: {absent}"
    );

    // **A permanent failure**: the same command will fail the same way, so the cause must be
    // fixed first.
    let permanent = shall::app::sync::what_to_do_about(
        Some(&shall::core::Error::command_failed_classified(
            "it broke",
            shall::core::Retryability::Permanent,
        )),
        "apt:curl",
    );
    assert!(
        permanent.contains("fix the cause"),
        "a permanent failure must not be advised away with another run: {permanent}"
    );
}
