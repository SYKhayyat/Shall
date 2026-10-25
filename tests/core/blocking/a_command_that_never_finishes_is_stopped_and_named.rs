use super::*;

/// **A script that never finishes is stopped, and the stop says so.**
///
/// The two doors that run user-authored code — an external `vars.<ext>` provider and
/// `sh()` in the Rhai stdlib — both sit before any package manager is asked, so an
/// unbounded wait there is an unbounded wait on `check`, `list`, `plan` and every
/// scheduled `sync`.
#[test]
fn a_command_that_never_finishes_is_stopped_and_named() {
    let mut cmd = if cfg!(windows) {
        let mut c = std::process::Command::new("cmd");
        c.args(["/c", "ping -n 30 127.0.0.1 > NUL"]);
        c
    } else {
        let mut c = std::process::Command::new("sh");
        c.args(["-c", "sleep 30"]);
        c
    };
    let started = std::time::Instant::now();
    let err = command_output_within(
        &mut cmd,
        "the wedged provider",
        Some(std::time::Duration::from_millis(300)),
    )
    .expect_err("a command that outlives its bound is not a result");
    assert_eq!(err.kind(), std::io::ErrorKind::TimedOut);
    assert!(
        err.to_string().contains("the wedged provider"),
        "the message must name what hung: {err}"
    );
    assert!(
        started.elapsed() < std::time::Duration::from_secs(10),
        "the bound did not bound anything"
    );
}
