use super::*;

/// A Windows exit code does not survive a Unix `ExitStatus`: `from_raw(code << 8)` reads
/// back as `code & 0xff`, so 1605 becomes 69 and -1978335189 becomes something else again.
/// These two tests ran green on Windows and red everywhere else for exactly that reason,
/// and nothing said so until the first push in 25 commits.
///
/// So the policy — which is plain data and true on every platform — is asserted directly,
/// and the `ensure_status` wiring around it is proved with a code the OS can carry.
#[test]
fn choco_msi_reboot_codes_are_benign_and_do_not_leak() {
    let choco = exit_policy::choco();
    let apk = exit_policy::apk();
    for code in [1605, 1614, 1618, 1641, 3010] {
        assert!(choco.is_benign(Some(code)), "choco {code} is a reboot code");
        assert!(
            !apk.is_benign(Some(code)),
            "apk borrowed choco's code {code}"
        );
    }
    assert!(!choco.is_benign(Some(1)), "1 is a plain choco failure");
    assert!(
        !choco.is_benign(None),
        "a killed command chose no code at all"
    );
    assert!(executor_for(exit_policy::choco())
        .ensure_status("choco", finished(1, "", ""))
        .is_err());
}
