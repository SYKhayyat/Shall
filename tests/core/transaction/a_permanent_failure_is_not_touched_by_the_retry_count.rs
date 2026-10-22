use super::*;

#[test]
fn a_permanent_failure_is_not_touched_by_the_retry_count() {
    // It never entered the retry loop a second time â€” `give_up` breaks on Permanent â€” so
    // seeing one here at all would mean something else changed. Pinned so it cannot.
    let e = Error::CommandFailed {
        message: "`scoop` failed: Couldn't find manifest".into(),
        retry: Retryability::Permanent,
        absent_name: true,
    };
    assert_eq!(
        falsify_transience(e, 3).retryability(),
        Retryability::Permanent
    );
}
