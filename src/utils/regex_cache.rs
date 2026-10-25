//! Compiled regexes, kept.
//!
//! `Regex::new` builds and optimises an automaton. Eleven places in this tree called it inside
//! the loop that used the result — per snapshot listed, per service row parsed, per package
//! line, per diagnostic rule per failure — so the same handful of patterns were recompiled
//! thousands of times in one command.
//!
//! Patterns that are literals in the source belong in a `Lazy` beside their use. This is for
//! the ones that come from **configuration**: a snapshot definition's `list_pattern`, a service
//! adapter's row pattern, an onboarder's extraction rule. Those cannot be `Lazy` — they are not
//! known until a file is read — but they are just as fixed for the length of a run.

use once_cell::sync::Lazy;
use regex::Regex;
use std::sync::Arc;

pub static CACHE: Lazy<dashmap::DashMap<String, std::result::Result<Arc<Regex>, String>>> =
    Lazy::new(dashmap::DashMap::new);

/// How many distinct patterns this cache will hold before it stops growing.
///
/// **The cache is per *process*, and until `watch` existed every process was one command.** A
/// command reads a bounded set of configuration — a handful of snapshot definitions, service
/// adapters and onboarder rules — so "never evicted" and "bounded" were the same thing and the
/// cache was free.
///
/// `watch` is the only caller that makes the process long-lived: it reconciles on a tick,
/// forever, re-reading configuration that a `--pull` may have just changed. There the two come
/// apart, and never-evicted becomes a slow leak — one automaton per pattern the config has ever
/// held, for as long as the daemon runs.
///
/// The number is generous on purpose. This tree's whole configuration surface is tens of
/// patterns; a machine that has legitimately seen a thousand distinct ones is a machine whose
/// config is being rewritten by something, and the cache going cold is the right outcome there
/// rather than unbounded memory.
pub const CAPACITY: usize = 1024;

/// This pattern, compiled once per process.
///
/// A pattern that will not compile is remembered as an error too: re-attempting a compile that
/// has already failed costs the same as one that succeeds, and the caller gets the same message
/// either way.
pub fn compiled(pattern: &str) -> std::result::Result<Arc<Regex>, String> {
    if let Some(hit) = CACHE.get(pattern) {
        return hit.clone();
    }
    let built = Regex::new(pattern).map(Arc::new).map_err(|e| e.to_string());
    // Cleared rather than evicted one-by-one, deliberately: an LRU needs a recency order, which
    // means a second structure and a write on every *read* — and reads are the hot path this
    // cache exists for. Dropping everything at the ceiling costs one recompile per live pattern,
    // once, on a boundary a normal command never reaches at all.
    if CACHE.len() >= CAPACITY {
        CACHE.clear();
    }
    CACHE.insert(pattern.to_string(), built.clone());
    built
}
