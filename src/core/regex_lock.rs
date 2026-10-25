//! `locks/regex.toml` — what each `re:` pattern expanded to (II.15).
//!
//! **The lock file IS the switch** (II.15). An entry means the pattern is frozen to that list;
//! no entry means it is asked again. There is no command either way: the first expansion
//! records itself, and deleting the entry is how you re-find (owner ruling, 2026-07-21).
//!
//! Freezing is the point, not a cache. `apt:re:^lib` matched 30,207 packages when it was
//! measured; a pattern re-expanded on every sync means the machine grows a package the day
//! someone else's upload happens to match, with nothing in your files changing and nothing to
//! review. The recorded list is in git, so what the pattern means is a diff.
//!
//! **Residual hole, accepted (II.15):** a package renamed out of the pattern silently drops
//! one package. One package, recoverable, and the snapshot has your back.

use crate::core::ledger::LockFile;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// `backend:pattern -> the names it matched`. Keyed by both because the same pattern against
/// two managers is two different questions with two different answers.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct RegexLock {
    #[serde(default)]
    expanded: BTreeMap<String, Vec<String>>,
}

pub fn key(backend: &str, pattern: &str) -> String {
    format!("{}:{}", backend, pattern)
}

impl LockFile for RegexLock {
    const WHAT: &'static str = "the regex lock";
}

impl RegexLock {
    pub fn path_in(locks_dir: &Path) -> PathBuf {
        locks_dir.join("regex.toml")
    }

    pub fn get(&self, backend: &str, pattern: &str) -> Option<&[String]> {
        self.expanded.get(&key(backend, pattern)).map(Vec::as_slice)
    }

    /// Record an expansion. Returns whether the file needs writing — an unchanged lock
    /// rewritten every run would make every sync a commit.
    pub fn record(&mut self, backend: &str, pattern: &str, mut names: Vec<String>) -> bool {
        names.sort();
        names.dedup();
        match self.expanded.get(&key(backend, pattern)) {
            Some(existing) if *existing == names => false,
            _ => {
                self.expanded.insert(key(backend, pattern), names);
                true
            }
        }
    }

    /// Forget every pattern no line declares any more, so the file cannot freeze an answer
    /// for a pattern that is gone and apply it again if the line comes back.
    pub fn retain_declared(&mut self, declared: &[String]) -> bool {
        let before = self.expanded.len();
        self.expanded.retain(|k, _| declared.iter().any(|d| d == k));
        self.expanded.len() != before
    }

    pub fn is_empty(&self) -> bool {
        self.expanded.is_empty()
    }
}
