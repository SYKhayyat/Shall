use super::*;

/// And a backend with no lock in the table is never made to wait for one, whatever its
/// failure happens to say.
#[test]
fn a_backend_with_no_manager_lock_backs_off_as_before() {
    let verdict = lock_wait_verdict(
        &lock_failure("`npm` failed: could not get lock"),
        "npm",
        Duration::from_secs(300),
        Duration::from_secs(300),
        &|_| panic!("npm has no manager lock, so nothing should have been asked"),
    );
    assert!(matches!(verdict, LockWait::Backoff), "{verdict:?}");
}
