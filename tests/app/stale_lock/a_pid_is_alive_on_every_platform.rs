//! **Does the answer to "is this pid alive" exist off Linux?**
//!
//! Session protection (`C16`, II.80) is a pid verdict: a registry row stamped by a session is kept
//! while that session's process is running. `pid_alive` read `/proc/<pid>`, which is Linux's
//! answer, and on macOS and Windows it therefore answered "no" about processes that were running —
//! so a package a live shell held was planned for removal, which is the failure the protection
//! exists for (#152).
//!
//! The witness in the suite was already there and was read as a flake for two days:
//! `a_live_session_keeps_its_package_and_the_skip_names_the_session` stamps the test's **own** pid
//! and asserts the preview names the session. It passed on `x86_64-apple-darwin` and failed on
//! `aarch64-apple-darwin` and MSVC, which is what an architecture-dependent syscall looks like
//! from the outside.
//!
//! These cases are the platform's own: they run wherever the suite runs, which is why the fix
//! cannot be verified on one machine alone — but "own pid", "reaped child" and "no such pid" are
//! answered by all three implementations, and each is a different syscall underneath.

use super::*;

/// The process running this test is alive. Trivially true, and that is the point: a `pid_alive`
/// that answers "no" about the caller cannot protect anything.
#[test]
fn the_running_process_is_alive() {
    let procs = ProcFs;
    assert!(
        procs.pid_alive(std::process::id()),
        "`pid_alive` said this test's own pid is dead, so every session-stamped package on this \
         platform is unprotected"
    );
}

/// A pid that has been reaped is not alive — the other direction, and the one that lets protection
/// relax once the shell is gone rather than keeping a package forever.
#[test]
fn a_reaped_child_is_not_alive() {
    let mut child = if cfg!(windows) {
        std::process::Command::new("cmd")
            .args(["/C", "exit", "0"])
            .spawn()
    } else {
        std::process::Command::new("sh")
            .args(["-c", "exit 0"])
            .spawn()
    }
    .expect("the fixture child should start");
    let pid = child.id();
    child.wait().expect("the fixture child should be reaped");

    let procs = ProcFs;
    // The OS is free to reuse a pid, but not within the microseconds between a `wait` and this
    // line on any machine this runs on — and a pid reused by a live process is genuinely alive.
    assert!(
        !procs.pid_alive(pid),
        "a child that exited and was reaped is still reported alive as {pid}, so a session's \
         protection outlives the shell that held it"
    );
}

/// A pid nothing could be using. This is what a stamp left by a killed process looks like.
#[test]
fn a_pid_no_process_holds_is_not_alive() {
    let procs = ProcFs;
    assert!(
        !procs.pid_alive(u32::MAX),
        "pid {} was reported alive, which no process can hold",
        u32::MAX
    );
}
