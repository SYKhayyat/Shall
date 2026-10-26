//! Can one comment say the same thing twice?
//!
//! Two of the four found by hand were merge artifacts rather than judgement calls, and both are
//! the same shape: **a contiguous run of comment lines containing a copy of itself.**
//!
//! - `backends/generic.rs` carried its three-line note about rpm's `%{name}` twice, on consecutive
//!   lines. A merge, and invisible on the page: the reader sees a paragraph that starts again.
//! - `model/resolve.rs` carried a **six-line `///` doc paragraph twice inside one doc comment**,
//!   which `cargo doc` renders as one doubled paragraph — so the published documentation for the
//!   set-math rule read as though it had been written twice, which is a different claim from the
//!   one it makes.
//!
//! **Why the rule is this narrow.** A comment that states its constraint once is doing its job; a
//! comment that states it twice states it once and owes a maintenance update twice, and the two
//! copies drift. That is the whole argument. It says nothing about a comment deliberately repeated
//! in *another* item, and two such sites were found by hand and left alone (`PLAN.md` #103):
//!
//! - `cli/args.rs` — the same four-line note on two different `--json` flags. **This one is right
//!   as it stands:** rustdoc renders each field's docs separately, so a self-contained field doc is
//!   the point rather than a copy.
//! - `backends/registry/os_native.rs` — the same three-line note in three registrar functions.
//!   **This one is a smell and is left for the owner**, because the constraint it states belongs to
//!   `with_manager_policy` and the three copies are three updates owed. Moving it is a judgement
//!   about where a constraint should live, not a merge to clean up.
//!
//! A gate that flagged either would be switched off, which is how a gate stops being a gate — so
//! they are named here instead, which is what makes the omission a decision rather than a gap.
//!
//! **Driven against a planted duplicate as its own control**, because a prohibition that cannot
//! fail reports green having examined nothing.

use std::path::{Path, PathBuf};

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

/// Contiguous runs of comment lines: the file line each run starts at, and its trimmed text.
///
/// **The line number is carried because a gate that prints "lines 19+4" without a file to open is
/// a gate that costs more to use than it saves.** That was this file's own first version, found by
/// running it.
fn comment_runs(src: &str) -> Vec<(usize, Vec<String>)> {
    let mut runs: Vec<(usize, Vec<String>)> = Vec::new();
    let mut current: Vec<String> = Vec::new();
    let mut start = 0usize;
    for (n, line) in src.lines().enumerate() {
        let t = line.trim();
        // `//`, `///` and `//!` all start a comment; `////` is a ruler, and a line that is only
        // slashes is not a statement about the code either way.
        if t.starts_with("//") && !t.starts_with("////") {
            if current.is_empty() {
                start = n + 1;
            }
            current.push(t.trim_start_matches('/').trim().to_string());
        } else if current.len() >= 2 {
            runs.push((start, std::mem::take(&mut current)));
        } else {
            current.clear();
        }
    }
    if current.len() >= 2 {
        runs.push((start, current));
    }
    runs
}

/// **One predicate, and it is the general one.** A run repeats itself when some stretch of its
/// lines appears twice inside it, which covers "the same paragraph immediately after itself" and
/// "the same paragraph twice with one line between" with the same code — two predicates for one
/// rule is the kind of duplication this gate exists to catch. Longest first, so the reported stretch is the largest one rather than the first
/// two lines of it.
fn repeated_anywhere<S: AsRef<str>>(run: &[S]) -> Option<(usize, usize)> {
    let lines: Vec<&str> = run.iter().map(AsRef::as_ref).collect();
    let n = lines.len();
    for len in (2..=n / 2).rev() {
        for start in 0..=(n - len * 2) {
            for shift in (start + len)..=(n - len) {
                if lines[start..start + len] == lines[shift..shift + len] {
                    return Some((start, len));
                }
            }
        }
    }
    None
}

#[test]
fn no_comment_says_the_same_thing_twice() {
    let mut files = Vec::new();
    rust_files(Path::new("src"), &mut files);
    assert!(files.len() > 50, "the scan is not walking the tree");

    let mut offenders: Vec<String> = Vec::new();
    let mut runs_seen = 0usize;
    for file in &files {
        let Ok(src) = std::fs::read_to_string(file) else {
            continue;
        };
        for (start, run) in comment_runs(&src) {
            runs_seen += 1;
            if let Some((at, len)) = repeated_anywhere(&run) {
                offenders.push(format!(
                    "{}:{}  {len} lines repeat, so one paragraph is there twice: {}",
                    file.display(),
                    start + at + 1,
                    run[at].chars().take(70).collect::<String>()
                ));
            }
        }
    }

    assert!(
        runs_seen > 500,
        "the scan matched {runs_seen} comment runs, which is too few to be reading this tree"
    );
    assert!(
        offenders.is_empty(),
        "a comment states its constraint once and owes a maintenance update twice, and the two \
         copies drift:\n  {}\n\n  A doubled `///` paragraph also reaches `cargo doc`, so the \
         published documentation reads as written twice.",
        offenders.join("\n  ")
    );
}

/// **The control, both directions.** The two shapes that were on the tree when this gate was
/// written must be caught, and four shapes that are merely adjacent must not be.
#[test]
fn the_sweep_catches_a_doubled_paragraph_and_spares_an_adjacent_one() {
    let paragraph = vec![
        "The operand is the argument that IS `{name}`, never one that merely contains it:",
        "dnf asks with `--queryformat %{name}`, where those six characters are rpm's own",
        "format language and substituting the package into them produces `%jq`.",
    ];
    let mut doubled = paragraph.clone();
    doubled.extend(paragraph.iter().cloned());
    assert_eq!(
        repeated_anywhere(&doubled),
        Some((0, 3)),
        "the sweep does not catch the duplicate this gate was written for"
    );

    let doc_paragraph = vec![
        "Apply a profile's set math to what it reaches (II.4).",
        "",
        "Order is fixed and stated in II.4: everything is gathered first, then narrowed by",
    ];
    let mut doubled_doc = doc_paragraph.clone();
    doubled_doc.extend(doc_paragraph.iter().cloned());
    assert_eq!(
        repeated_anywhere(&doubled_doc),
        Some((0, 3)),
        "the sweep does not catch a doubled `///` paragraph"
    );

    // Four distinct lines that share a first word, and a line repeated once on purpose.
    for adjacent in [
        vec![
            "// read this",
            "// and this",
            "// then this",
            "// and this other",
        ],
        vec![
            "// Why the loop is a loop:",
            "// because a second pattern is expected.",
            "// TODO",
        ],
    ] {
        assert_eq!(
            repeated_anywhere(&adjacent),
            None,
            "the sweep flags lines that merely look alike: {adjacent:?}"
        );
    }
    // One repeated line is emphasis, not a merge.
    assert_eq!(
        repeated_anywhere(&["// read the order below", "// and again"]),
        None,
        "a single repeated line is emphasis, and flagging it would cry wolf"
    );
}
