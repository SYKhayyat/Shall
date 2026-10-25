use super::*;

/// `dnf check-update` exits **100** when it finds updates. That is dnf's answer, and
/// unmarked it makes a successful update check indistinguishable from a broken one.
#[test]
fn dnf_reports_finding_updates_with_an_exit_code_that_is_not_a_failure() {
    assert!(
        dnf().is_benign(Some(100)),
        "exit 100 is `there are updates`, not `check-update failed`"
    );
    // And the codes that really are failures stay failures.
    assert!(!dnf().is_benign(Some(1)));
    assert!(!dnf().is_benign(None), "a signal kill chose no code");
}
