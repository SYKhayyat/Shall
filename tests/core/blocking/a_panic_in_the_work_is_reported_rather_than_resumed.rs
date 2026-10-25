use super::*;

/// A panic in the work is an error, not a panic in the caller: this runs inside a package
/// manager's retry loop, and an unwind there is a failure nobody can trace to its cause.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_panic_in_the_work_is_reported_rather_than_resumed() {
    let out = off_the_runtime(|| panic!("the archive was truncated")).await;
    assert!(out.is_err(), "a panicking job must not look like success");
}
