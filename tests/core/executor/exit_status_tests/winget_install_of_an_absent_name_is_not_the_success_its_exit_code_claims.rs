use super::*;

/// winget is choco's twin — the only other manager that forgives a non-zero exit — and it
/// forgives the same code for opposite events. `install` of a name that does not exist and
/// `uninstall` of something already gone both exit -1978335212; only the second is what was
/// asked for. Measured on Windows 11, winget's own wording tells them apart.
#[test]
fn winget_install_of_an_absent_name_is_not_the_success_its_exit_code_claims() {
    let winget = exit_policy::winget();
    assert!(
        winget.signals_failure(&ExitPolicy::haystack(
            b"No package found matching input criteria.",
            b""
        )),
        "nothing was installed"
    );
    assert!(
        !winget.signals_failure(&ExitPolicy::haystack(
            b"No installed package found matching input criteria.",
            b""
        )),
        "removing something already gone is the outcome that was wanted"
    );
}
