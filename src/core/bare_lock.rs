//! `locks/bare.HOST.toml` — which package manager an unpinned name resolved to (II.6).
//!
//! A line that does not pin one manager (`ripgrep`, or `apt,dnf:ripgrep`) is answered by
//! asking its candidates in order whether they have that name and taking the first yes (II.7
//! step 4). Unrecorded, that answer is re-derived every run against whatever is installed
//! *now* — so adding a package manager that sits higher in `priority` and happens to publish
//! the same name silently changes what an unedited line means. The record is the fix: asked
//! once, then the same answer until you say otherwise.
//!
//! **One file per host.** Which manager has `ripgrep` is a fact about a machine, and
//! `locks/` travels with the config to every machine that shares it. A single file would
//! hold whichever answer synced last: the Ubuntu box writes `apt`, the Fedora box overwrites
//! with `dnf`, and the two rewrite each other on every sync and collide in git forever. A
//! file per host means each machine writes only its own, every file commits cleanly, and
//! each machine still reproduces exactly what it had.
//!
//! **Deleting is how you unfreeze** (II.15's rule, applied here): an entry means frozen, no
//! entry means ask. `shall unlock backends` removes entries; so does an editor, because the file is
//! yours.
//!
//! One file per host rather than one per backend, unlike the rest of `locks/`: the fact
//! recorded is about a *name*, and a name that moves backends would otherwise be two writes —
//! a delete from one file and an insert into another — for one fact changing.

use crate::core::ledger::LockFile;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// `name -> backend`. A `BTreeMap` so the file is ordered and diffs cleanly in git.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct BareLock {
    #[serde(default)]
    resolved: BTreeMap<String, String>,
}

impl LockFile for BareLock {
    const WHAT: &'static str = "the bare-name lock";
}

impl BareLock {
    /// This machine's file. Every other machine sharing the config has its own, so a sync
    /// here can never overwrite the answer another host depends on.
    pub fn path_in(locks_dir: &Path) -> PathBuf {
        Self::path_for(locks_dir, &crate::config::Config::get_hostname())
    }

    /// Anything a hostname may legally contain but a filename should not becomes `_`, so a
    /// host called `../etc` writes inside `locks/` like every other host.
    ///
    /// **Distinct hosts stay distinct.** Folding every unsafe character to `_` collided
    /// `a.b`, `a b` and `a:b` onto one file — three machines, one resolution store, each
    /// clobbering the others' answers. The separator itself is escaped too: `%xx`, so the
    /// mapping stays injective and a filename still names exactly one host.
    pub fn path_for(locks_dir: &Path, host: &str) -> PathBuf {
        let mut safe = String::with_capacity(host.len());
        for b in host.bytes() {
            if b.is_ascii_alphanumeric() || b == b'-' {
                safe.push(b as char);
            } else if b == b'_' {
                // `_` is this scheme's own escape marker; escape it like anything else so
                // `a_b` and `a%5Fb`-style inputs cannot fold together either.
                safe.push_str("%5F");
            } else {
                safe.push_str(&format!("%{:02X}", b));
            }
        }
        let safe = if safe.is_empty() {
            "unknown".to_string()
        } else {
            safe
        };
        locks_dir.join(format!("bare.{}.toml", safe.to_lowercase()))
    }

    /// Forget one name, so it is asked again. Reports whether there was anything to forget.
    pub fn forget(&mut self, name: &str) -> bool {
        self.resolved.remove(name).is_some()
    }

    /// Forget everything.
    pub fn clear(&mut self) -> bool {
        let had = !self.resolved.is_empty();
        self.resolved.clear();
        had
    }

    /// Every frozen name and the manager it is frozen to, for `shall lock backends --list`.
    pub fn entries(&self) -> impl Iterator<Item = (&str, &str)> {
        self.resolved.iter().map(|(n, b)| (n.as_str(), b.as_str()))
    }

    /// The backend this name is frozen to, if any.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.resolved.get(name).map(String::as_str)
    }

    /// Record an answer. Returns whether the file needs writing — an answer that is already
    /// recorded is not a change, and rewriting an unchanged lock on every sync would make
    /// every run a git commit.
    pub fn record(&mut self, name: &str, backend: &str) -> bool {
        match self.resolved.get(name) {
            Some(existing) if existing == backend => false,
            _ => {
                self.resolved.insert(name.to_string(), backend.to_string());
                true
            }
        }
    }

    /// Forget every name that is no longer declared anywhere.
    ///
    /// Without this the file only grows, and a stale entry is worse than a missing one: it
    /// freezes an answer for a line that no longer exists, and would silently apply again if
    /// the name came back.
    pub fn retain_declared(&mut self, declared: &[String]) -> bool {
        let before = self.resolved.len();
        self.resolved.retain(|n, _| declared.iter().any(|d| d == n));
        self.resolved.len() != before
    }

    pub fn is_empty(&self) -> bool {
        self.resolved.is_empty()
    }
}
