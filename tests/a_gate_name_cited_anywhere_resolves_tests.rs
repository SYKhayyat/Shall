//! A gate's name, cited anywhere, must name a gate that exists — the twin of
//! `every_citation_of_a_why_entry_resolves_to_one`.
//!
//! That test answers one question for one corpus: does a `V.n` citation resolve to an entry in
//! `why.md`? `CLAUDE.md` makes that corpus mandatory reading, so a dangling citation there makes a
//! required read impossible. **Gate names are the other corpus and had no such test at all**,
//! which `PLAN.md` found while verifying #96: `every_gone_ok_tag_is_witnessed_somewhere` was cited
//! by `run-in-container.sh`, `why.md` and `spec/plan.md`, and existed in none of them — the check
//! was an inline block with an `echo` header, so a reader who grepped the citation found prose and
//! no gate.
//!
//! That instance was fixed by making the name greppable (`## every_gone_ok_tag_is_witnessed_somewhere`).
//! Nothing would have noticed the next one. This file is the noticing.
//!
//! **Citations are read out of backtick spans, with line breaks inside a span removed.** Both halves
//! are measurements rather than taste:
//!
//! - *Backticks.* Gate names are quoted as code wherever they are cited. Scanning bare prose
//!   instead finds the same names plus ordinary snake_case words, and the difference between the
//!   two candidate sets is entirely false positives.
//! - *Joining.* A name wrapped across a line is **not greppable**, which is the defect this file
//!   exists to catch, so a scanner that reports such a name as dangling is reporting its own
//!   line-breaking rather than the prose's. Joining recovers it. This found a real instance:
//!   `tests/a_writer_that_reaches_` + newline + `the_disk_goes_through_one_tests.rs`, fixed here —
//!   and it also found that the *other* wrapped name was missing an underscore outright, which
//!   joining does not recover and which had to be corrected in the prose.
//!
//! **Four forms count as defined**, because this corpus uses all four and a scanner that accepts
//! one produces a false report on every gate in the other three: a test `fn` name, a `mod` name in
//! `tests/main.rs`, a test file's stem, and that stem without its `_tests` suffix. The harness
//! scripts carry a fifth, the `## <name>` marker, which exists for exactly this reason.
//!
//! **What is exempt, and why each one is here rather than fixed.** An exemption list is this
//! repo's established shape (`lifecycle-floor.txt`'s dated excuses, `TOO_BIG_FOR_NOW`), and every
//! entry names a corpus where a gate name is cited as *history* — a gate that existed, was renamed
//! or deleted, and is named in a past-tense entry. Renaming those would falsify the record.
//! **The list may only shrink**, and the assertion below says so.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Where gate names are cited, and where they are defined.
const PROSE: &[&str] = &[
    "docs/spec/why.md",
    "docs/spec/plan.md",
    "docs/spec/bugs.md",
    "PLAN.md",
    "CLAUDE.md",
    "docs/SPEC.md",
    "AI_ISSUE_ROUTING.md",
];

/// Names cited as history, with the corpus that says so.
///
/// **Every entry is a citation of something that is not this repository's gate, or of a gate that
/// no longer exists.** Four shapes, and each was checked rather than pattern-matched:
///
/// 1. `docs/spec/bugs.md` — a dated log whose entries are marked FIXED, so a gate named there is
///    named in the past tense. Two of these are this repo's own retired tests.
/// 2. `test_*` — a Python suite belonging to another project, quoted in `spec/plan.md`. This
///    repository does not name a gate `test_*`; its gates open `a_`, `an_`, `the_`, `no_`,
///    `every_` or `unmeasured_`, or are a bare noun phrase.
/// 3. `an_escape_hatch_names_something_that_exists_tests` — a gate of this repo, deleted, and named
///    in a doc comment *because* it was deleted.
const EXEMPT: &[(&str, &str)] = &[
    // `docs/spec/bugs.md` is a dated log whose entries are marked FIXED, so a gate named there is
    // named in the past tense. Four of these are this repository's own retired tests and one is a
    // gate that has since been renamed — rewriting either would falsify the record.
    (
        "docs/spec/bugs.md",
        "an_escape_hatch_names_something_that_exists_tests",
    ),
    (
        "docs/spec/bugs.md",
        "enforce_refuses_without_opt_in_and_proceeds_with_it",
    ),
    (
        "docs/spec/bugs.md",
        "every_os_native_backend_sends_the_argv_its_manager_expects",
    ),
    (
        "docs/spec/bugs.md",
        "sources_are_repo_relative_and_forward_slashed",
    ),
    ("docs/spec/bugs.md", "test_e2e_sync_flow_hermetic"),
    ("docs/spec/bugs.md", "test_journal_self_healing_logic"),
    ("docs/spec/bugs.md", "test_journal_wal_healing_logic"),
    ("docs/spec/bugs.md", "test_parallel_task_isolation_wiring"),
    // The same retired gates, cited from the plan's history.
    (
        "docs/spec/plan.md",
        "an_escape_hatch_names_something_that_exists_tests",
    ),
    ("docs/spec/plan.md", "test_e2e_cross_backend_teleport"),
    ("docs/spec/plan.md", "test_journal_self_healing_logic"),
    (
        "docs/spec/plan.md",
        "test_teleport_api_consistency_on_missing_package",
    ),
    (
        "tests/the_exit_table_is_generated_not_retyped_tests.rs",
        "every_code_is_distinct_and_documented",
    ),
    // Gates of this repo, deleted, and named in a doc comment *because* they were deleted. The
    // second says so in as many words: "What was here before was X — it is deleted, not moved."
    (
        "tests/a_batch_of_installs_is_one_command_tests.rs",
        "test_e2e_sync_flow_hermetic",
    ),
    // A gate of this repo, deleted, and named in a doc comment *because* it was deleted.
    (
        "tests/a_removal_token_cannot_be_minted_tests.rs",
        "an_escape_hatch_names_something_that_exists_tests",
    ),
];

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    let path = repo().join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()))
}

fn rust_files(dir: &Path, into: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, into);
        } else if path.extension().is_some_and(|e| e == "rs") {
            into.push(path);
        }
    }
}

/// Every name that identifies a gate, in all four forms this corpus uses.
fn defined() -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut add = |name: &str| {
        out.insert(name.to_string());
        // The `_tests` suffix is cargo's file convention and the prose drops it, so a citation
        // reading `an_archive_cannot_wrap_past_the_size_bound` must resolve to the module
        // `an_archive_cannot_wrap_past_the_size_bound_tests`.
        if let Some(bare) = name.strip_suffix("_tests") {
            out.insert(bare.to_string());
        }
    };

    let mut files = Vec::new();
    rust_files(&repo().join("src"), &mut files);
    rust_files(&repo().join("tests"), &mut files);
    assert!(
        files.len() > 100,
        "the scan found {} Rust files, which means it is not scanning",
        files.len()
    );
    for f in &files {
        let Ok(text) = std::fs::read_to_string(f) else {
            continue;
        };
        for line in text.lines() {
            // A test `fn`, or a `mod` declaration in `tests/main.rs`. Both are how a gate is
            // introduced; neither is a comment, because a doc comment *citing* a name must not
            // count as defining it — that is the whole question.
            let trimmed = line.trim_start();
            for prefix in ["fn ", "pub fn ", "async fn ", "pub async fn ", "mod "] {
                if let Some(rest) = trimmed.strip_prefix(prefix) {
                    let name: String = rest
                        .chars()
                        .take_while(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '_')
                        .collect();
                    if name.len() >= 19 {
                        add(&name);
                    }
                }
            }
        }
        if f.parent().is_some_and(|p| p.ends_with("tests")) {
            if let Some(stem) = f.file_stem().map(|s| s.to_string_lossy().to_string()) {
                add(&stem);
            }
        }
    }

    // The `## <name>` marker the harness scripts carry for exactly this purpose, so a name in a
    // shell harness is greppable. Leading `#`s are stripped because the marker is a comment.
    let mut scripts = Vec::new();
    rust_files_sh(&repo().join("scripts"), &mut scripts);
    rust_files_sh(&repo().join("docker"), &mut scripts);
    for f in &scripts {
        let Ok(text) = std::fs::read_to_string(f) else {
            continue;
        };
        for line in text.lines() {
            let body = line.trim_start_matches('#').trim();
            if let Some(rest) = body.strip_prefix("## ") {
                let name: String = rest
                    .chars()
                    .take_while(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '_')
                    .collect();
                if name.len() >= 19 {
                    add(&name);
                }
            }
        }
    }
    out
}

fn rust_files_sh(dir: &Path, into: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files_sh(&path, into);
        } else if matches!(path.extension().and_then(|e| e.to_str()), Some("sh")) {
            into.push(path);
        }
    }
}

/// Every gate-shaped name quoted in `text`, whether or not it resolves.
///
/// **Backtick spans only**, because gate names are quoted as code wherever they are cited, and a
/// scan of bare prose finds the same names plus every snake_case word in the document.
///
/// **A line break inside a span is removed only where a single name was split**, and the rule for
/// that is the shape of the break rather than a guess: a wrapped identifier breaks *after* an
/// underscore, so the newline is dropped exactly when the character before it is `_` or the
/// character after it is. Joining every break instead merges the separate words of a wrapped
/// sentence into one invented name — which is how a first cut of this gate reported
/// `apt_like_coresetsorphan_dry_run`, its own artefact rather than a citation.
///
/// The shape is then a long, underscore-dense token, because a gate name here is a whole sentence
/// in snake_case and anything shorter or sparser is prose that happens to be quoted.
fn cited_in(text: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut rest = text;
    while let Some(open) = rest.find('`') {
        let after = &rest[open + 1..];
        let Some(close) = after.find('`') else {
            break;
        };
        let span = separate_fn_keyword(&join_split_names(&after[..close]));
        let chars: Vec<char> = span.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            if chars[i].is_ascii_lowercase() {
                let start = i;
                while i < chars.len()
                    && (chars[i].is_ascii_lowercase()
                        || chars[i].is_ascii_digit()
                        || chars[i] == '_')
                {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();
                if word.len() >= 25 && word.matches('_').count() >= 4 {
                    out.insert(word);
                }
                continue;
            }
            i += 1;
        }
        rest = &after[close + 1..];
    }
    out
}

/// Turn a `fn` in a quoted span into a separator, so quoting the function is not quoting a
/// longer name.
///
/// A doc comment that writes `` `fn a_default_policy_needs_a_direction_and_a_value` `` — which
/// several do, when the point is that the test *is* that function rather than a description of it
/// — otherwise reads as one identifier, `fna_default_policy_…`, which names nothing. Recognised at
/// a word boundary and looking past the space, so `async fn a_…` is caught and a gate whose own
/// name contains `fn` is left alone.
fn separate_fn_keyword(span: &str) -> String {
    let chars: Vec<char> = span.chars().collect();
    let mut out = String::with_capacity(chars.len() + 4);
    let mut i = 0;
    while i < chars.len() {
        let is_fn = chars[i] == 'f'
            && chars.get(i + 1) == Some(&'n')
            && !(i > 0 && (chars[i - 1].is_alphanumeric() || chars[i - 1] == '_'));
        if is_fn {
            let next = chars[i + 2..].iter().find(|c| !c.is_whitespace()).copied();
            if next.is_some_and(|c| c.is_ascii_lowercase() || c == '_') {
                out.push(';');
                i += 2;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// Skip the indentation and comment markers that begin a continuation line.
///
/// **The comment markers matter, and their absence was a false dangling citation.** A span inside
/// a Rust doc comment continues as `/// could_work`, and a scanner that skips only whitespace stops
/// at the first `/` — so `..._may_suggest_that_a_retry_` + newline + `/// could_work` reported as a
/// name truncated mid-way, which names nothing. A `/` or `*` at the *start of a line* is a comment
/// marker and never prose: a path cannot begin a line here without a newline in front of it, so
/// consuming the run cannot eat one. (Written without a literal example, because a sibling gate
/// reads every path-shaped token in a comment as a claim about this repository's tree.)
fn skip_continuation(chars: &[char], mut j: usize) -> usize {
    while matches!(
        chars.get(j),
        Some(' ') | Some('\t') | Some('\r') | Some('/') | Some('*')
    ) {
        j += 1;
    }
    j
}

/// Drop the line breaks that split one name, and whatever a continuation line opens with.
///
/// **Only where a single name was split**, and the rule is the shape of the break rather than a
/// guess: a wrapped identifier breaks *after* an underscore. Joining every break instead merges the
/// separate words of a wrapped sentence into one invented name — which is how a first cut of this
/// gate reported `apt_like_coresetsorphan_dry_run`, its own artefact rather than a citation.
fn join_split_names(span: &str) -> String {
    let chars: Vec<char> = span.chars().collect();
    let mut out = String::with_capacity(chars.len());
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\n' || chars[i] == '\r' {
            let j = skip_continuation(&chars, i + 1);
            if out.chars().last() == Some('_') {
                i = j;
                continue;
            }
            out.push(' ');
            i += 1;
            continue;
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// Every exempt name, as a `file::name` pair, for the message a reader has to act on.
fn exempt_pairs() -> BTreeSet<String> {
    EXEMPT
        .iter()
        .map(|(file, name)| format!("{file}::{name}"))
        .collect()
}

#[test]
fn every_gate_name_cited_anywhere_names_a_gate_that_exists() {
    let defined = defined();
    assert!(
        defined.len() > 500,
        "only {} gate names were found — the scan is broken, not the corpus",
        defined.len()
    );

    // **Rust comments are scanned too**, because that is where two of the three real dangling
    // citations were: a doc comment pointing a reader at a gate that does not exist is the same
    // dead link as one in prose, and `PLAN.md`'s instance was cited from a shell harness.
    let exempt = exempt_pairs();
    let mut dangling: Vec<String> = Vec::new();
    for file in PROSE {
        for name in cited_in(&read(file)) {
            if !defined.contains(&name) {
                dangling.push(format!("{file}::{name}"));
            }
        }
    }
    let mut files = Vec::new();
    rust_files(&repo().join("src"), &mut files);
    rust_files(&repo().join("tests"), &mut files);
    for f in &files {
        let rel = f
            .strip_prefix(repo())
            .unwrap_or(f)
            .to_string_lossy()
            .replace('\\', "/");
        // **This file is not scanned.** It necessarily cites every name it exempts, so including
        // it would mean the gate can never be green — and a gate that is always red is a gate that
        // gets deleted rather than fixed.
        if rel == "tests/a_gate_name_cited_anywhere_resolves_tests.rs" {
            continue;
        }
        for name in cited_in(&std::fs::read_to_string(f).unwrap_or_default()) {
            if !defined.contains(&name) {
                dangling.push(format!("{rel}::{name}"));
            }
        }
    }

    let dangling: Vec<String> = dangling
        .into_iter()
        .filter(|d| !exempt.contains(d))
        .collect();
    assert!(
        dangling.is_empty(),
        "these names are cited as gates and name no gate:\n  {}\n\n\
         A reader who greps one finds prose and no check, which is the same dead link \
         `every_citation_of_a_why_entry_resolves_to_one` refuses for a `V.n`. Either the gate was \
         renamed and the citation was not, or the citation was always aspirational.\n\n\
         A name that is genuinely historical belongs in `EXEMPT` above, with its corpus — an \
         exemption list is only honest if it may only shrink.",
        dangling.join("\n  ")
    );

    // **The list may only shrink**, and this is what stops it becoming a place to park anything
    // awkward. Same rule as `UNCITED_CEILING` in the `V.n` twin, and for the same reason: an
    // exemption that can be added to freely is an exemption nobody reads.
    //
    // **Asserted both ways**, because a ceiling that may only fall is not a gate unless something
    // notices when it stops falling — which is the second half of `the_pile_of_rationale_...`.
    let stale: Vec<String> = EXEMPT
        .iter()
        .map(|(file, name)| format!("{file}::{name}"))
        .filter(|pair| {
            let (file, name) = pair.split_once("::").unwrap_or(("", ""));
            !std::fs::read_to_string(repo().join(file))
                .map(|t| cited_in(&t).contains(name))
                .unwrap_or(false)
        })
        .collect();
    assert!(
        stale.is_empty(),
        "these exemptions no longer match anything, so the corpus changed underneath them and the \
         list should be re-read rather than trusted:\n  {}\n\nAn exemption that names nothing is \
         not an exemption; it is a name that has stopped being load-bearing and will be read as \
         permission by the next person who wants one.",
        stale.join("\n  ")
    );
}
