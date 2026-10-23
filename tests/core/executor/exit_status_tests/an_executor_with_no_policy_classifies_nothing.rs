use super::*;

/// An executor nobody gave a policy fails every non-zero exit and classifies nothing —
/// the behaviour every backend had before policies existed.
#[test]
fn an_executor_with_no_policy_classifies_nothing() {
    let plain = CommandExecutor::new(true, false);
    assert!(plain
        .ensure_status("choco", finished(3010, "", ""))
        .is_err());
    assert!(plain.ensure_status("apt", finished(0, "", "")).is_ok());
    let err = plain
        .ensure_status("apt", finished(100, "", "E: Unable to locate package nope"))
        .expect_err("non-zero must fail");
    assert_eq!(err.retryability(), Retryability::Unknown);
}
