use super::*;

/// The sentence, because the branch is only half the harm.
#[test]
fn a_permanent_failure_is_not_advised_as_if_nobody_had_looked() {
    let e = boxed(Error::CommandFailed {
        message: "`nixos-rebuild` failed (exit 4): Unable to autolaunch a dbus-daemon".to_string(),
        retry: Retryability::Permanent,
        absent_name: false,
    });
    let advice = kept_line_advice(
        why_kept(&e),
        "nixos:hello",
        std::path::Path::new("modules/imperative.txt"),
    );
    assert!(
        !advice.contains("Nothing classified the failure above"),
        "the advice for a Permanent failure is the Unclassified sentence:{}{advice}",
        NEWLINE
    );
    assert!(
        advice.contains("permanent"),
        "the advice never says what Shall decided about it:{}{advice}",
        NEWLINE
    );
}
