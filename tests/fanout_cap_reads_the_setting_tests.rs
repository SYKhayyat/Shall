//! Every concurrency cap in the tree reads a config field (AU9).
//!
//! `planner.rs` states the rule in its own words — *"`max_parallel`, and a cap that ignores the
//! setting is a cap the user cannot move"* — and one fan-out ignored it: `guard.rs` hard-coded
//! `.buffer_unordered(8)` for the per-backend OS-essential query, which is on **every removal
//! path**, which is exactly where somebody who has turned the parallelism down most wants it
//! honoured.
//!
//! A rule written in a comment beside one fan-out is a rule about that fan-out. This is the same
//! rule asked of all of them, which is the only form that survives the next one being added.
//!
//! **Why a source scan.** Same reason as `removal_guard_enumeration_tests.rs`: the finding is
//! about a site that exists and is not covered, and no behaviour can enumerate the sites nobody
//! tested. Adding a literal cap anywhere in `src/` fails this until it reads a setting.
//!
//! **And the scan had a hole of its own, which is how `PLAN.md` #84's fan-out got through it.**
//! The test for "hard-codes its width" was *"the argument starts with a digit"* — which is a way
//! of saying *a literal*, and a literal is only one of the two ways a width can ignore the
//! setting. The other is **the width of the thing being fanned out**: `.buffered(members.len()
//! .max(1))` in the transaction engine's prior-state read reads no literal and no config field,
//! and opens one listing per member of the batch — four hundred of them, each of which can spawn
//! a manager process, at the one call that runs *before anything is touched*. So the rule is now
//! the one the test's own name states: **no fan-out is as wide as its input.**

use std::path::{Path, PathBuf};

/// The stream combinators that take a width. `buffered` and `buffer_unordered` are futures';
/// `Semaphore::new` is tokio's, and it is a cap by another name.
const CAPS: &[&str] = &["buffer_unordered(", "buffered(", "Semaphore::new("];

/// Why a width argument is not a cap, or `None` when it is one.
///
/// Two shapes, and the second was added because a site slipped past the first:
///
/// - **a literal** — `buffer_unordered(8)`. The setting exists; this ignores it.
/// - **the input's own length** — `buffered(members.len().max(1))`. No literal and no config
///   field, so it read as a cap to a scan looking for literals, and it is the *opposite* of one:
///   the wider the batch, the wider the fan-out, which is the shape of a bug that only appears on
///   the machines with the biggest batches.
///
/// A named local (`cap`, `width`) is a cap: the caller computed it, and the scan cannot see what
/// from — so the rule is about what the *argument* is, and a local is an answer rather than a
/// length. `max(1)` around either is a clamp, not a width.
fn why_not_a_cap(rest: &str) -> Option<&'static str> {
    if rest.contains(".len()") {
        return Some("the width is the collection's own length");
    }
    if rest.starts_with(|c: char| c.is_ascii_digit()) {
        return Some("the width is a literal");
    }
    None
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn no_fan_out_in_the_tree_hard_codes_its_width() {
    let mut files = Vec::new();
    rust_files(Path::new("src"), &mut files);
    assert!(
        files.len() > 50,
        "only {} source files found; the walk is broken, not the tree",
        files.len()
    );

    let mut offenders: Vec<String> = Vec::new();
    let mut found_any = 0usize;

    for file in &files {
        let Ok(text) = std::fs::read_to_string(file) else {
            continue;
        };
        for (n, line) in text.lines().enumerate() {
            let trimmed = line.trim_start();
            // Its own doc comment naming the mistake is not the mistake.
            if trimmed.starts_with("//") {
                continue;
            }
            for cap in CAPS {
                let Some(rest) = trimmed.split_once(cap).map(|(_, r)| r) else {
                    continue;
                };
                found_any += 1;
                if let Some(why) = why_not_a_cap(rest) {
                    offenders.push(format!(
                        "{}:{}  {}  — {why}",
                        file.display(),
                        n + 1,
                        trimmed.trim_end()
                    ));
                }
            }
        }
    }

    assert!(
        found_any > 3,
        "found only {} fan-outs at all; the scan matches nothing and would pass over anything",
        found_any
    );
    assert!(
        offenders.is_empty(),
        "these fan-outs are not as wide as the user configured. A cap that ignores the setting \
         is a cap the user cannot move, and a width taken from the collection being fanned out \
         ignores it in the direction that hurts most — the bigger the batch, the wider the \
         fan-out, and each of these can spawn a manager process:\n  {}",
        offenders.join("\n  ")
    );
}

/// **The predicate's own tests, both directions, because the hole it had was a predicate that
/// only looked where it had already looked.** The first case is the line that got through; the
/// second is a literal, which is what the scan used to catch and must still catch; and the rest
/// are the shapes a correct caller writes, because a gate that fires on those is a gate that gets
/// switched off.
#[test]
fn a_width_is_a_cap_unless_it_is_a_literal_or_the_input_s_own_length() {
    assert_eq!(
        why_not_a_cap("members.len().max(1))"),
        Some("the width is the collection's own length"),
        "the scan must catch the line `PLAN.md` #84 is about, or it is the same hole twice"
    );
    assert_eq!(why_not_a_cap("8)"), Some("the width is a literal"));
    assert_eq!(why_not_a_cap("config.max_concurrent.max(1))"), None);
    assert_eq!(why_not_a_cap("config.network_parallel.max(1))"), None);
    assert_eq!(why_not_a_cap("self.config.network_parallel.max(1))"), None);
    assert_eq!(why_not_a_cap("width)"), None);
    assert_eq!(why_not_a_cap("cap)"), None);
}

/// **The other half of #84, which no width check can see.** Lowering a width is safe; *changing
/// which combinator carries it* is not, and the difference is invisible to the scan above. The
/// batch's prior-state read returns `Vec<Prior>` that is indexed by member position
/// (`priors[i]`, three sites): it is not a set of answers, it is one answer per slot, and
/// `buffer_unordered` would fill those slots in completion order. Every `priors[i]` would then be
/// some other package's history, `rollback` would restore the wrong thing, and nothing would fail
/// — no compile error, no test, just a rollback that is confidently wrong.
///
/// So the rule is pinned where it can be: **the positional read keeps `buffered`.** A source
/// check, for the same reason as the scan above — the mis-index is invisible until the day a
/// rollback fires on a real machine.
const POSITIONAL_READ: &str = "src/core/transaction.rs";

#[test]
fn a_result_that_is_indexed_by_position_stays_ordered() {
    let src = std::fs::read_to_string(Path::new(POSITIONAL_READ)).expect("transaction.rs");
    // The read is positional only if something indexes its result by position; without this the
    // rule below would be asserting a constraint on a value nothing depends on.
    assert!(
        src.contains("priors[i]"),
        "{POSITIONAL_READ} no longer indexes the prior read by position; if that is deliberate, \
         delete this test with the reasoning rather than leaving it green"
    );
    let width_line = src
        .lines()
        .find(|l| {
            let t = l.trim();
            t.starts_with(".buffered(") || t.starts_with(".buffer_unordered(")
        })
        .expect("the prior read opens a fan-out");
    assert!(
        width_line.trim_start().starts_with(".buffered("),
        "{POSITIONAL_READ} fans the positional prior read out with `{}`. Its result is indexed by \
         member position (`priors[i]`), so completion order is the wrong order: every answer would \
         land in another package's slot and `rollback` would restore the wrong thing, with \
         nothing to fail.",
        width_line
            .trim()
            .trim_start_matches('.')
            .split('(')
            .next()
            .unwrap_or("?")
    );
}

/// The control for the line above, in both directions.
#[test]
fn the_ordered_rule_would_notice_a_reordering() {
    let bad = "                .buffer_unordered(config.max_concurrent.max(1))";
    assert!(
        !bad.trim_start().starts_with(".buffered("),
        "the rule does not notice `buffer_unordered`, so the test above is asserting nothing"
    );
    let good = "                .buffered(config.max_concurrent.max(1))";
    assert!(good.trim_start().starts_with(".buffered("));
}

/// Where each `Semaphore` lives, and why that scope is the run.
///
/// **The sibling assertion, and the one the gate above cannot make.** `no_fan_out_hard_codes_its
/// _width` checks a cap's *value*; nothing checked its *scope*. `StateResolver`'s `remote_gate`
/// read `network_parallel` perfectly and was constructed inside `StateResolver::new` — and
/// `App::resolver()` was not memoised, so it minted a fresh resolver, and a fresh gate, at every
/// one of its 34 call sites. Every one of those is sequential within its command, which is the
/// only reason the cap held; the first concurrent caller would have multiplied it silently, with
/// nothing to notice.
///
/// `core::ratelimiter` had already written the lesson down one directory over: *"`Arc<OnceLock<_>>`
/// rather than `OnceLock` inside a clone: the cell is what the clones share, so two backends
/// holding copies of one quota still hold ONE quota. **A per-clone cell would silently double
/// every limit here.**"*
///
/// A source scan cannot prove a semaphore is shared. What it can do is refuse a new one that
/// nobody has said who owns — which is the question that went unasked.
const SEMAPHORE_OWNERS: &[(&str, &str)] = &[
    (
        "src/app/context.rs",
        "the `App`'s `remote_gate`, and the `App` is the run. Handed to every `StateResolver` \
         this run builds and carried into every `Machinery` it hands out, so `network_parallel` \
         is a cap on this process's remote lookups rather than on one short-lived struct.",
    ),
    (
        "src/app/sync/resolver.rs",
        "the fallback for a `StateResolver::new` built without an `App` — `verbs::check`'s free \
         functions, `app::repl`, `app::run`. Each of those is one resolver used sequentially \
         within one command, so its own gate IS the run's; a caller that has an `App` goes \
         through `App::resolver`, which hands over the shared one instead.",
    ),
    (
        "src/app/search.rs",
        "one search command, one fan-out over the backends it asks, built where the fan-out is \
         and dropped with it. There is no second search running concurrently to share it with, \
         and a process-wide search gate would mean two `shall search` invocations in one \
         process rationed each other.",
    ),
    (
        "src/core/transaction.rs",
        "the execution semaphore for one graph run, derived from `max_concurrent`. A \
         transaction is the unit the cap is about: two transactions in one process are two \
         separate sets of package work, and the `DataLock` already serialises writers so they \
         cannot overlap anyway.",
    ),
];

/// **Every cap says who owns it.**
///
/// See `SEMAPHORE_OWNERS`. This does not prove sharing — nothing static can — but a semaphore
/// built somewhere nobody has explained is exactly the shape R5 was, and the failure message
/// asks the question that would have caught it.
#[test]
fn every_semaphore_says_what_scope_it_caps() {
    let mut files = Vec::new();
    rust_files(Path::new("src"), &mut files);

    let mut built_in: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut lines: Vec<(String, String)> = Vec::new();

    for file in &files {
        let Ok(text) = std::fs::read_to_string(file) else {
            continue;
        };
        // A fixture's semaphore lives for one test, which is not the scope this is about.
        let body = match text.find("\n#[cfg(test)]") {
            Some(i) => &text[..i],
            None => text.as_str(),
        };
        let relative = file.to_string_lossy().replace('\\', "/");
        for line in body.lines() {
            let t = line.trim_start();
            if t.starts_with("//") || !t.contains("Semaphore::new(") {
                continue;
            }
            built_in.insert(relative.clone());
            lines.push((relative.clone(), t.trim_end().to_string()));
        }
    }

    assert!(
        !built_in.is_empty(),
        "no `Semaphore::new` found anywhere; the scan matches nothing and would pass over \
         anything"
    );

    let named: std::collections::BTreeSet<&str> =
        SEMAPHORE_OWNERS.iter().map(|(f, _)| *f).collect();

    for (file, reason) in SEMAPHORE_OWNERS {
        assert!(
            reason.len() >= 100,
            "{file}'s entry is {} characters. Say what object owns the semaphore and why that \
             object is the run — \"it's fine\" is what R5 would have said.",
            reason.len()
        );
        assert!(
            built_in.contains(*file),
            "SEMAPHORE_OWNERS names {file}, which no longer builds one. Delete the entry: an \
             explanation of something that is gone reads as an explanation of something that \
             is there."
        );
    }

    let unnamed: Vec<&String> = built_in
        .iter()
        .filter(|f| !named.contains(f.as_str()))
        .collect();
    if !unnamed.is_empty() {
        let mut msg = String::from(
            "these files build a `Semaphore` and SEMAPHORE_OWNERS does not say what scope it \
             caps:",
        );
        for f in &unnamed {
            msg.push_str(&format!("\n    {f}"));
            for (site, line) in &lines {
                if site == *f {
                    msg.push_str(&format!("\n        {line}"));
                }
            }
        }
        msg.push_str(
            "\n\nAnswer this, in a sentence: what object holds it, and is that object the run? \
             A cap constructed inside a short-lived struct is a cap nobody shares — every clone \
             of that struct gets the full allowance, and the number the user set stops meaning \
             what they meant. If the answer is `App`, take it from `App::remote_gate` instead \
             of building a new one.",
        );
        panic!("{msg}");
    }
}
