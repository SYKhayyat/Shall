use super::*;

#[test]
fn a_held_lock_is_transient_and_a_missing_name_is_not() {
    assert_eq!(
        apt().retryability(&ExitPolicy::haystack(
            b"",
            b"E: Could not get lock /var/lib/dpkg/lock-frontend"
        )),
        Retryability::Transient
    );
    assert_eq!(
        apt().retryability(&ExitPolicy::haystack(
            b"",
            b"E: Unable to locate package nosuchpkg"
        )),
        Retryability::Permanent
    );
    assert_eq!(
        apt().retryability(&ExitPolicy::haystack(
            b"",
            b"E: Something nobody classified"
        )),
        Retryability::Unknown
    );
}
