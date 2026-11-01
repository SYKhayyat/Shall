use super::*;

/// The half that must not regress. A dropped network or a held lock means you did mean
/// it, and the line stays so a retry works.
#[test]
fn a_transient_or_unclassified_failure_keeps_the_line() {
    for retry in [Retryability::Transient, Retryability::Unknown] {
        let e = boxed(Error::CommandFailed {
            message: "`apt` failed: Could not get lock /var/lib/dpkg/lock".to_string(),
            retry,
            absent_name: false,
        });
        assert!(!says_a_name_is_absent(&e), "withdrew on {retry:?}");
        assert_eq!(absent_command_message(&e), None, "withdrew on {retry:?}");
    }
}
