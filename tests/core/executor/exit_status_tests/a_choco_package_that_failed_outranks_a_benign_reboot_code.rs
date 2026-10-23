use super::*;

/// Chocolatey forces its exit code to 1 when a package failed *only if nothing else already
/// set one* — so a dependency that asks for a reboot leaves 3010 standing over a package
/// that never installed. 3010 is benign here, which made a 10-of-11 install a success and
/// the harness say `choco installed bat for real` about a `bat` that was not there.
///
/// The count sentence is what knows, and choco prints it only when something failed.
/// Asserted on the policy rather than through a fabricated `ExitStatus`, for the reason the
/// test above gives: 3010 does not survive a Unix status, and 0 does.
#[test]
fn a_choco_package_that_failed_outranks_a_benign_reboot_code() {
    let choco = exit_policy::choco();
    assert!(
        choco.signals_failure(&ExitPolicy::haystack(
            CHOCO_UNINSTALL_ABSENT.as_bytes(),
            b""
        )),
        "choco said 0/1 and 1 failed"
    );
    assert!(
        choco.signals_failure(&ExitPolicy::haystack(
            b"Chocolatey installed 10/11 packages. 1 packages failed.",
            b""
        )),
        "a dependency took the package down with it"
    );
    assert!(
        !choco.signals_failure(&ExitPolicy::haystack(CHOCO_INSTALL_OK.as_bytes(), b"")),
        "a clean 11/11 install says nothing about failing"
    );
    assert!(choco.is_benign(Some(3010)), "still a reboot, still benign");
    assert!(executor_for(exit_policy::choco())
        .ensure_status("choco", finished(0, CHOCO_UNINSTALL_ABSENT, ""))
        .is_err());
    assert!(executor_for(exit_policy::choco())
        .ensure_status("choco", finished(0, CHOCO_INSTALL_OK, ""))
        .is_ok());
}
