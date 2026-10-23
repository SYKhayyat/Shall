use super::*;

/// The point of the whole change: the failure the retry loop reads carries its own
/// verdict, so it never has to read the message back.
#[test]
fn a_failure_carries_its_retryability() {
    let apt = executor_for(exit_policy::apt());
    let permanent = apt
        .ensure_status("apt", finished(100, "", "E: Unable to locate package nope"))
        .expect_err("exit 100 is a failure");
    assert_eq!(permanent.retryability(), Retryability::Permanent);

    let transient = apt
        .ensure_status(
            "apt",
            finished(100, "", "E: Could not get lock /var/lib/dpkg/lock-frontend"),
        )
        .expect_err("exit 100 is a failure");
    assert_eq!(transient.retryability(), Retryability::Transient);
    assert!(matches!(transient, Error::CommandFailed { .. }));
}
