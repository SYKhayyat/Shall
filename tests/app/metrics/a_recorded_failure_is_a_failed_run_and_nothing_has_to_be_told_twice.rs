use super::*;

/// **The assertion nothing in the tree could make, because the only writer had no callers.**
///
/// `Status:` read a field nobody wrote, so it was the constant `SUCCESS` and `DEGRADED` was
/// unreachable code. A run where every package failed printed `Status: SUCCESS` with a
/// failure mark under each task line, and the same run under `--quiet` printed nothing at
/// all (B1). The state is now derived from the operations that were actually recorded, so a
/// recorded failure is a degraded run by construction.
#[test]
fn a_recorded_failure_is_a_failed_run_and_nothing_has_to_be_told_twice() {
    let m = MetricsCollector::new();
    assert!(!m.had_failures(), "a run with no operations has not failed");

    m.record_operation(Recorded {
        name: "cowsay",
        backend: "apt",
        started: Utc::now(),
        success: true,
        error: None,
        retries: 0,
        bytes_downloaded: 0,
        batch_size: 1,
    });
    assert!(
        !m.had_failures(),
        "an operation that succeeded is not a failure"
    );

    m.record_operation(Recorded {
        name: "shall-no-such-package-zzz",
        backend: "apt",
        started: Utc::now(),
        success: false,
        error: Some("E: Unable to locate package".into()),
        retries: 0,
        bytes_downloaded: 0,
        batch_size: 1,
    });
    assert!(
        m.had_failures(),
        "a package that failed to install must make the run a failed one — this is the \
         fact `Status: DEGRADED` and the exit code both read"
    );
}
