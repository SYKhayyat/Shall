//! **Is the session that owns this row still running?** (`C16`, `II.80`, `V.232`)
//!
//! `shall shell` stamps the packages it installs with the session's id, and while that id is in
//! `active_session_id` the planner treats those rows as the session's — so a sync does not remove
//! a package out from under a running shell. The protection is correct while the session lives.
//!
//! **It outlives the session when the session is killed.** Teardown clears the field and removes
//! the packages; a `SIGKILL`, a closed laptop lid or a torn-down container never reaches teardown
//! at all, so the field stays set forever, every row it stamped stays protected, and nothing
//! checked whether the session behind them still existed.
//!
//! **The ruling is that a stale session stops protecting and reaps nothing.** Three reasons:
//!
//! 1. **Reaping is a deletion, and a deletion must be previewable.** If staleness deleted
//!    packages, `shall sync` — a command that converges toward your manifests — would start
//!    removing software on the strength of a pid check. Relaxing the protection instead leaves
//!    the ordinary drift rules to propose the removal through the planner, behind the removal
//!    guard, where `shall plan` shows it and the user can refuse it.
//! 2. **The two ways to be wrong are not symmetric.** Answering "stale" about a live session
//!    removes a running shell's packages; answering "live" about a dead one leaves the bug in
//!    place. So the check may only be trusted to *relax a protection*, never to *authorise a
//!    deletion*.
//! 3. **Relaxing is idempotent by construction.** It cannot half-happen and cannot depend on when
//!    it ran.
//!
//! **Unanswerable means alive.** A container whose process table the owner cannot read, a shared
//! data directory, a platform with no `/proc` — each of those treats the row as live, which is
//! the status quo. That means this cannot fix such a host; saying so is better than a check that
//! guesses in the direction that deletes something.

use std::path::Path;

pub use crate::app::stale_lock::Processes;

/// What a session stamp can be found to be.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionLiveness {
    /// The session's process is there, or the question could not be answered. Protected.
    Live,
    /// The session's process is gone. Protected no longer — and not removed either.
    Stale,
}

impl SessionLiveness {
    /// Whether a row stamped by this session is still the session's to keep.
    pub fn protects(&self) -> bool {
        matches!(self, Self::Live)
    }
}

/// The stamp a session writes: its id, and the pid that may still be running.
///
/// The id alone cannot answer this — a uuid is not a process. The pid is what makes the question
/// askable at all, and it is written **beside** the id rather than derived from it so an
/// unparseable or older stamp answers `None` rather than guessing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionStamp {
    pub session_id: String,
    pub pid: u32,
}

impl SessionStamp {
    /// Parse `shell-<uuid>` + pid, as the shell writes them.
    ///
    /// Deliberately strict: anything that does not parse is `None`, and `None` is treated as live
    /// by [`liveness`]. A lenient parser here would be a lenient parser deciding whether to stop
    /// protecting something.
    pub fn parse(session_id: &str, pid_text: &str) -> Option<Self> {
        let pid: u32 = pid_text.trim().parse().ok()?;
        if session_id.trim().is_empty() {
            return None;
        }
        Some(Self {
            session_id: session_id.to_string(),
            pid,
        })
    }
}

/// **Is the session behind this stamp still there?**
///
/// `None` when the stamp does not parse — and that is `Live`, which is the whole fail-open rule.
pub fn liveness(stamp: Option<&SessionStamp>, processes: &dyn Processes) -> SessionLiveness {
    match stamp {
        // No parseable stamp: no evidence the session is gone, so it keeps its protection.
        None => SessionLiveness::Live,
        Some(stamp) if processes.pid_alive(stamp.pid) => SessionLiveness::Live,
        Some(_) => SessionLiveness::Stale,
    }
}

/// The pid a session recorded, read from the file the shell writes.
///
/// Kept separate from [`SessionStamp::parse`] so the file shape can change without the rule
/// changing: this is I/O, that is the decision.
pub fn read_stamp(session_dir: &Path, session_id: &str) -> Option<SessionStamp> {
    let text = std::fs::read_to_string(session_dir.join(format!("{session_id}.pid"))).ok()?;
    SessionStamp::parse(session_id, &text)
}
