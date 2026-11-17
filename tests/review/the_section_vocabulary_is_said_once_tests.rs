//! **A vocabulary with one list must be said from that list in every sentence that lists it.**
//!
//! `#186`. `check --help` named seven sections and the command accepted ten, and the three it
//! omitted were the three most recently added — `durability` among them, the section `#74` shipped
//! this release. `Section::vocabulary()` exists **for this exact purpose**, with a doc comment
//! saying so:
//!
//! ```text
//! /// The one place the vocabulary is read from, so the parser and the error cannot disagree
//! /// about what is legal.
//! ```
//!
//! The parser and the error cannot disagree. The **help text**, a third reader of the same list,
//! could and did, because it is a `#[arg]` doc-comment and a doc-comment cannot call a function.
//!
//! `shall export` had the same defect twice over, and that half is `#191`'s: `Format::all()` has
//! held five entries, and both sentences that listed the formats — the `--format` help and the
//! `--stdout` refusal — said four, missing `nix`. It landed as one table, `Format::NAMES`, with
//! `WORDS` and `HELP` read from it and a test holding the help against the table in both
//! directions; the refusal now names `Format::WORDS` rather than a fourth copy. **This file keeps
//! the half that had no table to read from.**
//!
//! **This is not a check that the help is good.** It is the narrow one: every word the product can
//! accept appears in the sentence that tells a person what it accepts. The list is read from
//! `Section::ALL` rather than restated here, so adding an eleventh section reds this by name instead
//! of waiting for somebody to remember.

use std::path::{Path, PathBuf};

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn args_rs() -> String {
    std::fs::read_to_string(repo().join("src/cli/args.rs")).unwrap_or_default()
}

/// The doc-comment lines that sit above a field, so a sentence can be found by the field it
/// describes rather than by the words in it.
fn doc_above(within: &str, field: &str) -> String {
    let source = args_rs();
    let all: Vec<&str> = source.lines().collect();
    let opening = format!("    {within} {{");
    let from = all
        .iter()
        .position(|l| *l == opening)
        .unwrap_or_else(|| panic!("no `{within}` variant in src/cli/args.rs — this gate is stale"));
    let at = all
        .iter()
        .enumerate()
        .skip(from)
        .find(|(_, l)| l.trim_start().starts_with(field) && l.trim_end().ends_with(','))
        .map(|(i, _)| i)
        .unwrap_or_else(|| panic!("no field `{field}` under `{within}` in src/cli/args.rs"));

    let mut start = at;
    while start > from
        && (all[start - 1].trim_start().starts_with("///")
            || all[start - 1].trim_start().starts_with("#["))
    {
        start -= 1;
    }
    all[start..at].join("\n")
}

/// **The `One of:` sentence itself**, not the paragraph around it.
///
/// This exists because the first version of this gate asked whether the *doc block* contained each
/// word — and the paragraph explaining the fix says "`nix` was missing here", so deleting `nix`
/// from the list left the gate green. Measured: the mutation that removes `| nix` passed. A gate
/// that a prose sentence can satisfy is a gate about prose.
fn one_of_sentence(doc: &str) -> String {
    let mut out: Vec<&str> = Vec::new();
    let mut inside = false;
    for line in doc.lines() {
        let body = line.trim_start().trim_start_matches("///").trim();
        if body.starts_with("One of:") {
            inside = true;
        } else if inside && body.is_empty() {
            break;
        }
        if inside {
            out.push(body);
        }
    }
    assert!(
        !out.is_empty(),
        "no `One of:` sentence in this doc block — the gate is stale, not passing:\n{doc}"
    );
    out.join(" ")
}

/// **Every section `check` accepts is named in the sentence that lists the sections.**
#[test]
fn the_section_vocabulary_is_said_once() {
    use shall::app::check::Section;

    let help = one_of_sentence(&doc_above("Check", "section: Option<String>"));
    let missing: Vec<&str> = Section::ALL
        .iter()
        .map(|s| s.as_str())
        .filter(|name| !help.contains(name))
        .collect();

    assert!(
        missing.is_empty(),
        "`check --help` does not name {missing:?}, and `check {missing:?}` works.\n\n\
         The help is a `#[arg]` doc-comment, so it cannot call `Section::vocabulary()` — which is \
         why it drifted from ten sections to seven with nothing watching. The list is short and the \
         section names are stable; keep the sentence in step with `Section::ALL` and this test will \
         tell you when it is not.\n\n\
         The help says:\n{help}"
    );
}
