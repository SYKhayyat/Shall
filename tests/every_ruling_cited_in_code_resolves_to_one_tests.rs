//! A ruling cited in code names a ruling that exists.
//!
//! `CLAUDE.md` reserves `D* W* K* N* T* U*` for register entries: an ID with one of those
//! prefixes is a question whose answer is the owner's, so a builder who meets one **stops and
//! asks**. `Q*` (the production-readiness round and the grading rounds after it) and `S*` (the
//! Shall bug log) are the two series the corpus cites next; every `V.n` is already covered by
//! `why_entries_are_attached_to_something_tests`, and this is the twin for the rest.
//!
//! **There is one venue, and it is `docs/spec/`.** `docs/SPEC.md` defines it: *"the
//! specification itself is in `spec/`, one file per part"* — `decisions.md` is the register
//! and `bugs.md` is the S-series, which is why `id_namespaces_do_not_collide_tests` already
//! treats that whole directory as the register and its annotations. A new
//! `docs/rulings.md` would be a *second* venue, which is the disease this repo keeps finding
//! rather than the cure for it.
//!
//! **What was found the day this was written — twelve, and every one a citation that had been
//! lying for a year.** Ten were BUILDER's own work orders: `docs/BUILDER.md` numbered its
//! orders `W1`–`W43`, which collided head-on with the register's reserved `W*`, and renumbered
//! them to `B*` when `id_namespaces_do_not_collide_tests` shipped. The document was fixed;
//! the twelve places in `src/` and `tests/` that cite those orders were not, so `W35`, `W36`,
//! `W37`, `W38`, `W41`, `W43`, `W21`, `W25` and `W29` had been reading as *"stop and ask the
//! owner"* ever since. The other two were `S22` and `S23` — real bugs, fixed, and never given
//! a row in `bugs.md`, so the three places naming the failure class cited nothing.
//!
//! **`EXEMPT` holds the citations that are history and must not be re-pointed.**
//! `id_namespaces_do_not_collide_tests` is *about* that collision, so quoting the pre-rename
//! `W1`–`W43` series is its subject rather than a citation. Renaming them would falsify the
//! record of the thing the file exists to pin. **The list may only shrink:** every entry is
//! asserted to still name a citation that exists, so an entry whose corpus stopped quoting it
//! must be deleted rather than left as a permission.
//!
//! And why the scan reads comments and not only them: a citation in a `//!` header is a
//! citation. Scanning all text is also why the vocabulary is an explicit list rather than
//! "every capital letter and digits" — `E0432` is a rustc error code and `B867` is a colour,
//! and neither is a ruling.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// This file, which quotes rulings as examples in its own header and so cannot scan itself —
/// the same rule `a_citation_in_a_comment_still_points_at_its_claim_tests` states for its own.
const SELF: &str = "every_ruling_cited_in_code_resolves_to_one_tests.rs";

/// The ruling vocabulary: the register's reserved prefixes plus the two series the corpus
/// cites that the reservation list does not name.
///
/// `D W K N T U` is what `CLAUDE.md` reserves; `Q` and `S` are the production-readiness rulings
/// and the Shall bug log. `V` is deliberately absent — `why_entries_are_attached_to_something_
/// tests` already resolves that one, and a second oracle for the same citation is a
/// contradiction waiting to happen.
const VOCABULARY: [&str; 8] = ["D", "W", "K", "N", "T", "U", "Q", "S"];

/// Where a ruling is defined, and the shape its definition takes.
///
/// One venue in two files, which is the shape the specification itself describes:
/// `decisions.md` holds the register (`## U1`), `bugs.md` holds the S-series (`| **S87** |`).
const REGISTER: &str = "docs/spec/decisions.md";
const BUG_LOG: &str = "docs/spec/bugs.md";

/// Cited as history, with the corpus that says so. **May only shrink.**
///
/// Every entry is a pre-2026-08-08 BUILDER work-order number quoted by the file that exists to
/// record the collision with the register's `W*`. Those numbers were reissued as `B*`; these
/// quotes are the record of what they used to be.
const EXEMPT: [(&str, &str); 4] = [
    ("W1", "tests/id_namespaces_do_not_collide_tests.rs"),
    ("W29", "tests/id_namespaces_do_not_collide_tests.rs"),
    ("W43", "tests/id_namespaces_do_not_collide_tests.rs"),
    ("W9a", "tests/id_namespaces_do_not_collide_tests.rs"),
];

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    let path = repo().join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()))
}

/// Every ruling ID the venue defines: a `## ID` register heading, or a `| **ID** |` bug-log row.
fn defined_by_the_venue() -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for line in read(REGISTER).lines() {
        if let Some(id) = line.strip_prefix("## ") {
            let id = id.trim();
            if let Some(letters) = id.chars().next().map(|c| c.to_string()) {
                let rest = &id[letters.len()..];
                if VOCABULARY.contains(&letters.as_str())
                    && !rest.is_empty()
                    && rest
                        .chars()
                        .all(|c| c.is_ascii_digit() || c.is_ascii_lowercase())
                {
                    out.insert(id.to_string());
                }
            }
        }
    }
    for line in read(BUG_LOG).lines() {
        if let Some(rest) = line.strip_prefix("| **") {
            let id: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric())
                .collect();
            if id.starts_with('S')
                && id[1..]
                    .chars()
                    .all(|c| c.is_ascii_digit() || c.is_ascii_lowercase())
                && id.len() > 1
            {
                out.insert(id);
            }
        }
    }
    out
}

/// Every ruling ID the code cites, paired with the file that cites it.
///
/// The file matters: an exemption is a permission granted to **one corpus**, and an ID-level
/// exemption would excuse the same citation anywhere — which is exactly how `W43` could stay
/// stale in `tests/dry_run_every_verb_tests.rs` while the pair quoted by the file that is
/// *about* the collision went on being excused.
fn cited_by_code() -> BTreeSet<(String, String)> {
    let mut files = Vec::new();
    for dir in ["src", "tests"] {
        collect(&repo().join(dir), &mut files);
    }
    let mut out = BTreeSet::new();
    for f in files {
        if f.file_name().and_then(|n| n.to_str()) == Some(SELF) {
            continue;
        }
        let rel = f
            .strip_prefix(repo())
            .unwrap_or(&f)
            .to_string_lossy()
            .replace('\\', "/");
        let Ok(text) = std::fs::read_to_string(&f) else {
            continue;
        };
        let chars: Vec<char> = text.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let letter = chars[i];
            if VOCABULARY.contains(&letter.to_string().as_str())
                && (i == 0 || !chars[i - 1].is_alphanumeric())
                && chars.get(i + 1).is_some_and(|c| c.is_ascii_digit())
            {
                let mut j = i + 1;
                while chars.get(j).is_some_and(|c| c.is_ascii_digit()) {
                    j += 1;
                }
                // One optional letter suffix: `W9a`, `Y7a`, `D3b`.
                if chars.get(j).is_some_and(|c| c.is_ascii_lowercase())
                    && !chars.get(j + 1).is_some_and(|c| c.is_alphanumeric())
                {
                    j += 1;
                }
                // **And a boundary behind it too,** or `W32Time` — the Windows service — is read
                // as a citation of a work order called `W32`. What follows the digits must not
                // continue the identifier.
                if chars
                    .get(j)
                    .is_some_and(|c| c.is_alphanumeric() || *c == '_')
                {
                    i += 1;
                    continue;
                }
                out.insert((chars[i..j].iter().collect(), rel.clone()));
                i = j;
                continue;
            }
            i += 1;
        }
    }
    out
}

fn collect(dir: &Path, into: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, into);
        } else if path.extension().is_some_and(|e| e == "rs") {
            into.push(path);
        }
    }
}

/// **The instrument before the verdict.** A scan over no files, or one whose matcher stopped
/// matching, reports a clean tree and is indistinguishable from a clean tree.
#[test]
fn the_scan_reads_the_tree_and_the_venue_defines_what_it_looks_like() {
    let mut files = Vec::new();
    collect(&repo().join("src"), &mut files);
    collect(&repo().join("tests"), &mut files);
    assert!(
        files.len() > 100,
        "the scan read {} Rust files, which means it is not scanning",
        files.len()
    );

    let defined = defined_by_the_venue();
    assert!(
        defined.len() > 100,
        "the venue yielded only {} definitions — the parser is broken, not the file",
        defined.len()
    );

    // The two definition shapes the venue actually uses, and the control that says the matcher
    // cannot invent one: a number with no entry is not a definition.
    assert!(defined.contains("U27"), "a `## U27` register heading");
    assert!(defined.contains("S87"), "a `| **S87** |` bug-log row");
    assert!(
        !defined.contains("U999"),
        "a number the venue never defined must not read as a definition, or this gate would \
         pass on a file that had been emptied"
    );
}

#[test]
fn every_ruling_cited_in_code_resolves_to_one() {
    let defined = defined_by_the_venue();
    let cited = cited_by_code();

    let mut surprising: Vec<String> = cited
        .iter()
        .filter(|(id, corpus)| !defined.contains(id) && !EXEMPT.contains(&(id.as_str(), corpus)))
        .map(|(id, corpus)| format!("{id} (in {corpus})"))
        .collect();
    surprising.sort();
    assert!(
        surprising.is_empty(),
        "these rulings are cited in `src/` or `tests/` and defined in neither `{}` nor `{}`:\n  \
         {}\n\nEither the citation's number is wrong, or the ruling it names was never written \
         down. A citation a builder cannot read is a stop-and-ask that goes unanswered.",
        REGISTER,
        BUG_LOG,
        surprising.join(", ")
    );
}

/// **The other half of the ratchet, and the half that rots quietly.** An exemption list is a
/// set of standing permissions, so every entry must still name a citation that exists. An
/// entry left behind after its citation was re-pointed is a permission nobody granted — which
/// is how the next dead citation gets waved through.
#[test]
fn every_exemption_still_names_a_citation_that_exists() {
    let cited = cited_by_code();
    let mut stale = Vec::new();
    for (id, corpus) in EXEMPT {
        let pair = (id.to_string(), corpus.to_string());
        if !cited.contains(&pair) {
            stale.push(format!("`{id}` in {corpus}"));
        }
    }
    assert!(
        stale.is_empty(),
        "these exemptions name a citation the code no longer makes, so the exemption is a \
         permission nobody granted — delete it in this change:\n  {}",
        stale.join("\n  ")
    );
}
