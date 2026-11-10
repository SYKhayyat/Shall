// tests/review/a_dated_essay_stays_only_while_a_living_document_cites_it.rs
//
// **`#41` / `C24`: which records stay, and what keeps one.**
//
// `Y21` cut `docs/archive/`, `docs/spec/proposals/`, `history.md` and `INEFFICIENCIES.md` on the
// reasoning that a record organised by *session* is for a reader who has sessions, and an agent
// does not. The cut did not reach the `AUDIT-`/`GRADE-`/`HANDOFF-` essays, which are the same kind
// of file: twenty-six of them, 622 KB, and twenty-three cited by nothing a maintainer reads.
//
// The test for which stay is a citation **from a living document**, and it has to be transitive or
// it does not work at all: HANDOFFs cite their predecessors, so a first pass that kept anything
// cited by anything kept sixteen of twenty-six. The family was keeping itself alive. Asking instead
// which essays the register, the bug list or a review test cites kept three.
//
// This file keeps that property: a new dated essay has to be cited by something durable, or it is
// a record nobody reads and it belongs in git history rather than in the tree.
use std::path::PathBuf;
use std::process::Command;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// The dated essays still in `docs/`, as bare file names.
///
/// Named by date so a `README-2026-08-12.md` in `docs/attic` — which is a *manual*, not an essay —
/// does not count.
fn dated_essays() -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(repo().join("docs")) else {
        return Vec::new();
    };
    let mut out: Vec<String> = entries
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let stem = name.strip_suffix(".md")?;
            let is_essay = ["AUDIT-", "GRADE-", "HANDOFF-"]
                .iter()
                .any(|p| stem.starts_with(p));
            // A date follows the dash: `GRADE-2026-08-13`.
            let has_date = stem
                .split('-')
                .nth(1)
                .is_some_and(|y| y.len() == 4 && y.chars().all(|c| c.is_ascii_digit()));
            (is_essay && has_date).then_some(name)
        })
        .collect();
    out.sort();
    out
}

/// Every file that matches `needle`, excluding the essay itself and every other essay.
///
/// Excluding the other essays is the whole point: their mutual citations are what the transitive
/// version of this question had to see through.
fn cited_by_a_living_document(essay: &str) -> Vec<String> {
    let out = Command::new("grep")
        .args([
            "-rl",
            "--include=*.md",
            "--include=*.rs",
            "--include=*.sh",
            essay,
            ".",
        ])
        .current_dir(repo())
        .output()
        .expect("grep runs");
    let essays = dated_essays();
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|l| l.trim_start_matches("./").to_string())
        .filter(|p| !p.is_empty())
        .filter(|p| p != &format!("docs/{essay}"))
        .filter(|p| {
            let base = p.rsplit('/').next().unwrap_or(p);
            !essays.iter().any(|e| e == base)
        })
        .collect()
}

/// **Every dated essay left in the tree is cited by a living document.**
///
/// The three that survive qualify: `AUDIT-2026-08-18-concurrency` and `GRADE-2026-08-12` from
/// `docs/spec/decisions.md` and `docs/spec/bugs.md`, `GRADE-2026-08-13` from the register and from
/// a review test that cites finding `F10` by name.
#[test]
fn every_dated_essay_is_cited_by_something_durable() {
    let orphans: Vec<(String, Vec<String>)> = dated_essays()
        .into_iter()
        .map(|e| {
            let citers = cited_by_a_living_document(&e);
            (e, citers)
        })
        .filter(|(_, citers)| citers.is_empty())
        .collect();

    assert!(
        orphans.is_empty(),
        "these dated essays are cited by no living document, only possibly by each other:\n{}\n\
         A record organised by session is for a reader who has sessions (`V.240`). Keep it if \
         something durable sends a reader to it — add the citation — and otherwise cut it: the file \
         stays in git, and `docs/README.md` says which commit and which `git checkout` brings it \
         back.",
        orphans
            .iter()
            .map(|(e, _)| format!("  docs/{e}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// **The sweep still sees essays, and the family has not grown back.**
///
/// A sweep that matched nothing would pass the test above by finding nothing. Twenty-six existed
/// before the cut and three after; a number climbing back toward twenty-six means the family is
/// being re-created, which is the drift this pair exists to catch.
#[test]
fn the_dated_essay_family_has_not_grown_back() {
    let essays = dated_essays();

    assert!(
        !essays.is_empty(),
        "the sweep found no dated essays at all. Three were kept because durable documents cite \
         them, so a zero means this test can no longer see the files it is judging"
    );
    assert!(
        essays.len() <= 4,
        "the tree holds {} dated essays: {essays:?}. `C24` cut this family from twenty-six to \
         three. A new one needs a durable citation to stay, and the ceiling here is one above the \
         three so that adding a cited record is possible without raising it",
        essays.len()
    );
}

/// **`docs/README.md` says how to get one back.**
///
/// The ruling's own condition: a record is cut from the tree and not from history. If the note
/// stops naming the commit and the command, a reader who wants a deleted essay has to know git well
/// enough to find it without help — which is the cost the cut was supposed to avoid paying twice.
#[test]
fn the_removal_note_names_the_way_back() {
    let readme = std::fs::read_to_string(repo().join("docs/README.md")).expect("docs/README.md");

    assert!(
        readme.contains("git checkout"),
        "`docs/README.md` no longer says how to recover a cut essay. The essays are in git; a \
         reader should not have to discover that on their own"
    );
    assert!(
        readme.contains("recoverable from git"),
        "the note no longer says the essays are recoverable, so a reader has no way to learn that \
         cutting a record from the tree is not cutting it from history"
    );
}
