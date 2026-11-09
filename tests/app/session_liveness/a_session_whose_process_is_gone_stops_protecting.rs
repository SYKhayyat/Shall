//! **A stale session stops protecting, and reaps nothing** (`C16`, II.80, V.232).
//!
//! The four arms, and why each one is here rather than left to the caller:
//!
//! * **alive → protects.** The status quo, asserted so a change that stopped protecting *every*
//!   session would be caught. That change would remove a running shell's packages, which is the
//!   failure this whole rule exists to prevent.
//! * **gone → stops protecting.** The bug: a killed session left its rows protected forever.
//! * **unanswerable → protects.** A container whose process table the owner cannot read, a shared
//!   data directory, a platform with no `/proc`. This is the fail-open rule, and it is the one
//!   that decides the design: the check may relax a protection, never authorise a deletion, so
//!   not knowing has to land on the protecting side.
//! * **and no arm deletes.** The ruling is "reaps nothing" — so there is deliberately no function
//!   here that removes a package, and the test that would have written one is this file's absence
//!   of a `remove` call.
use super::*;

/// **A running session keeps its packages.** The direction that must not regress.
#[test]
fn a_live_session_protects_its_rows() {
    let table = Table { alive: vec![4242] };
    assert_eq!(
        liveness(Some(&stamp("shell-a", 4242)), &table),
        SessionLiveness::Live,
        "the session's process is running, so its packages are in use and must stay"
    );
    assert!(liveness(Some(&stamp("shell-a", 4242)), &table).protects());
}

/// **A killed session stops protecting its rows** — and that is all it does.
///
/// Nothing here removes anything: the planner sees the row, and the next `shall plan` proposes the
/// removal through the ordinary drift rules, behind the removal guard, where the user can read it
/// and refuse it.
#[test]
fn a_session_whose_process_is_gone_stops_protecting() {
    let table = Table { alive: vec![] };
    let verdict = liveness(Some(&stamp("shell-a", 4242)), &table);

    assert_eq!(
        verdict,
        SessionLiveness::Stale,
        "pid 4242 is not in the process table, so the session that stamped these rows is gone"
    );
    assert!(
        !verdict.protects(),
        "a dead session must not keep protecting packages — that is the bug this rule closes"
    );
}

/// **An unreadable or unparseable stamp protects.** No evidence is not evidence of staleness.
///
/// This is the arm that makes the rule safe to ship. It is also the arm that means the fix cannot
/// help a host where the question is unanswerable — which is a cost worth stating out loud rather
/// than discovering as a deletion.
#[test]
fn an_unanswerable_stamp_protects() {
    let table = Table { alive: vec![] };

    assert_eq!(
        liveness(None, &table),
        SessionLiveness::Live,
        "no stamp at all: the session may be running on a host whose pid we never recorded, so \
         the rows keep their protection"
    );

    // A stamp whose pid is absent from the process table is `Stale`, which is the *answerable*
    // case. An unanswerable one is `None` above, and a stamp that does not parse is also `None`.
    assert_eq!(
        SessionStamp::parse("shell-a", ""),
        None,
        "an empty pid file must not parse — a lenient parser here would be a lenient parser \
         deciding whether to stop protecting something"
    );
    assert_eq!(SessionStamp::parse("", "4242"), None);
    assert_eq!(SessionStamp::parse("shell-a", "not a pid"), None);
    assert_eq!(
        SessionStamp::parse("shell-a", " 4242\n"),
        Some(stamp("shell-a", 4242)),
        "whitespace around the pid is the file's shape, not a malformed stamp"
    );
}

/// **A row with no session at all is nobody's transient and is untouched by this rule.**
///
/// The `None` session id is the common case — most managed rows were never installed by a shell —
/// and a rule that treated a missing id as "the session is gone" would stop protecting every
/// ordinary package's... which is fine for those (they were never protected), but the point here
/// is that the *planner* must not reach this rule for them at all. Asserted at the decision
/// boundary: there is no stamp to look for, so there is nothing to be stale about.
#[test]
fn a_row_with_no_session_has_no_stamp_to_read() {
    let dir = std::env::temp_dir().join(format!("shall-liveness-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("the session directory");

    assert_eq!(
        read_stamp(&dir, "shell-never-installed"),
        None,
        "a session with no pid file answers `None`, which the liveness rule treats as protected"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

/// **The stamp round-trips the way the shell writes it**, so the rule is exercised against the
/// real file rather than against a hand-built struct.
#[test]
fn a_stamp_round_trips_through_the_file_the_shell_writes() {
    let dir = std::env::temp_dir().join(format!("shall-liveness-rt-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("the session directory");
    std::fs::write(dir.join("shell-abc.pid"), std::process::id().to_string())
        .expect("the pid file");

    let read = read_stamp(&dir, "shell-abc").expect("the stamp parses");
    assert_eq!(read.session_id, "shell-abc");
    assert_eq!(read.pid, std::process::id());

    // And this process is alive, so the real answer for a real stamp is `Live`. The fake table is
    // not used here on purpose: this asserts the file shape and the pid, not the verdict.
    assert_eq!(read, stamp("shell-abc", std::process::id()));

    let _ = std::fs::remove_dir_all(&dir);
}
