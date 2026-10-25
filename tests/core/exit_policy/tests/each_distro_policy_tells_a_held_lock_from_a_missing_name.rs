use super::*;

/// Every distro manager answers both questions, or the fast-fail only helps Debian.
#[test]
fn each_distro_policy_tells_a_held_lock_from_a_missing_name() {
    let cases = [
        (
            dnf(),
            "Another app is currently holding the yum lock",
            "No match for argument: nope",
        ),
        (
            pacman(),
            "error: unable to lock database",
            "error: target not found: nope",
        ),
        (
            apk(),
            "ERROR: temporary error (try again later)",
            "ERROR: unable to select packages:",
        ),
        (
            brew(),
            "curl: (28) Operation timed out",
            "Error: No available formula with the name \"nope\"",
        ),
    ];
    for (policy, transient, permanent) in cases {
        assert_eq!(
            policy.retryability(&ExitPolicy::haystack(b"", transient.as_bytes())),
            Retryability::Transient,
            "not transient: {transient}"
        );
        assert_eq!(
            policy.retryability(&ExitPolicy::haystack(b"", permanent.as_bytes())),
            Retryability::Permanent,
            "not permanent: {permanent}"
        );
    }
}
