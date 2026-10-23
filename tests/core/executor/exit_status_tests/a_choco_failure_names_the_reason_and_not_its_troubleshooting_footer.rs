use super::*;

/// Chocolatey prints six lines of generic troubleshooting advice after *any* failure, and
/// the reason above it. With no vocabulary to pick lines by, `detail_for_user` fell back to
/// the tail — so every choco failure ever reported was those six lines, and the sentence
/// that said what went wrong was inside the `(13 more line(s))` nobody could read.
#[test]
fn a_choco_failure_names_the_reason_and_not_its_troubleshooting_footer() {
    let shown = CommandExecutor::detail_for_user(&exit_policy::choco(), CHOCO_UNINSTALL_ABSENT);
    assert!(
        shown.contains("Cannot uninstall a non-existent package"),
        "the reason is missing:\n{shown}"
    );
    assert!(
        !shown.contains("dependencies for a reason"),
        "the footer crowded out the reason:\n{shown}"
    );
}
