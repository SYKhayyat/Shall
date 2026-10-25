use super::*;

/// **The distinction N-1 was about.** A command failure can be permanent and be about a
/// name that plainly exists. Reading permanence as absence withdrew declarations for
/// packages that were installed; reading it as the *only* road to absence left every
/// manager with no policy wedging the config.
#[test]
fn a_permanent_failure_about_a_name_that_exists_never_withdraws() {
    let cases = [
        // helm refusing an unsignable plugin source, and refusing one already installed.
        ("plugin already exists", exit_policy::helm()),
        (
            "plugin source does not support verification",
            exit_policy::helm(),
        ),
        // A crate that is real and simply ships no program.
        ("error: there are no binaries", exit_policy::cargo()),
        // nimble: the package exists, the `@version=` on the line does not.
        ("Error: Version not found", exit_policy::nimble()),
        // scoop declining to remove what is not on the machine says nothing about the
        // bucket, and a failed uninstall must never delete the declaration.
        ("ERROR 'jq' isn't installed.", exit_policy::scoop()),
    ];
    for (output, policy) in cases {
        assert_eq!(
            policy.retryability(&shall::core::ExitPolicy::haystack(output.as_bytes(), b"")),
            Retryability::Permanent,
            "not permanent, so this case does not test the distinction: {output}"
        );
        assert!(
            !policy.names_an_absent_package(&shall::core::ExitPolicy::haystack(
                output.as_bytes(),
                b""
            )),
            "read as a missing name, so a declaration would be withdrawn over: {output}"
        );
    }
}
