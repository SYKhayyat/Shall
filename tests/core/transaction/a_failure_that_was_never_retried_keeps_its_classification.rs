use super::*;

#[test]
fn a_failure_that_was_never_retried_keeps_its_classification() {
    // The control. Downgrading on the first attempt would delete the distinction entirely
    // and make every transient failure Unknown, which is the opposite of the fix.
    let out = falsify_transience(transient("`apt` failed: Could not get lock"), 1);
    assert_eq!(out.retryability(), Retryability::Transient);
    assert!(!out.to_string().contains("did not change"));
}
