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

/// **The class, not the one value (#169).** `u32::MAX` was the reported case, and it was one of a
/// family: every pid above `i32::MAX` wraps when it is cast to a platform's signed `pid_t`, and a
/// wrapped value reaching `kill` is a wildcard (`kill(-1, 0)` means *every* process) rather than a
/// name — so the whole family answered "alive" and a dead session's protection never relaxed.
///
/// The three values are the boundaries of the cast rather than three arbitrary numbers:
///
/// - `i32::MAX` is the largest value that still fits a `pid_t`, and it **stays** answerable — a
///   refusal here would be a second bug in the safe-looking direction.
/// - `i32::MAX + 1` is the first value the truncation can reach, which is the exact edge.
/// - `u32::MAX` is what a `Some(u32::MAX)` stamp actually holds, and it truncates to `-1`.
///
/// The middle one is the reason this test is not a duplicate of the one above: a fix that special-
/// cases `u32::MAX` passes that test and fails this one.
#[test]
fn a_pid_above_the_signed_range_is_not_alive() {
    let procs = ProcFs;
    for pid in [i32::MAX as u32 + 1, u32::MAX - 1, u32::MAX] {
        assert!(
            !procs.pid_alive(pid),
            "pid {pid} is above the largest pid any platform can hold and was reported alive, so \
             a stamp left by a killed session protects its packages for ever"
        );
    }
}

/// The other side of the same boundary, and it is here so the fix cannot be "refuse large numbers".
/// `i32::MAX` fits a `pid_t`, so it is a question the platform is allowed to answer — and the
/// answer is whatever the machine says, not a refusal invented above the syscall.
#[test]
fn the_largest_pid_that_fits_is_still_asked_about() {
    let procs = ProcFs;
    // Not asserted to be alive: nothing is guaranteed to hold it. Asserted to be *asked*, which is
    // what fails if the boundary is set one step too low.
    let _ = procs.pid_alive(i32::MAX as u32);
    assert!(
        a_pid_can_name_a_process(1) && a_pid_can_name_a_process(i32::MAX as u32),
        "the boundary rejected a pid the platform's own `pid_t` can hold"
    );
}
