//! **Every top-level command is classified in the audit, exactly once** (#189).
//!
//! `docs/AUDIT-2026-11-17-command-classification.md` is the evidence `#166`'s model rests on: it
//! reads what each command does and files it as input, output, read-only or a named exception. The
//! document is only worth that much if it is **complete**, and nothing checked that it was.
//!
//! The record shows why that matters: `purge-undeclared` — the most destructive command in the
//! binary, the only one whose effect is "remove everything I did not mention" — was not on the
//! issue's list of commands needing a close look, and was found by doing the audit. A command
//! added after the audit is unclassified by *omission*, silently, and the next reader of the model
//! has no way to know.
//!
//! So this asks the program which commands it has and the document which ones it classified, and
//! requires the two to be the same set with no repeats. Aliases count as the command they name.
//!
//! **What this cannot see.** It compares *sets*, not the classification of any single row — a
//! command filed as `output` that is really `input` passes. Classifying is reading, and this is a
//! floor under the reading, not a substitute for it.

use std::collections::{BTreeMap, BTreeSet};

fn repo() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

const AUDIT: &str = "docs/AUDIT-2026-11-17-command-classification.md";

fn audit() -> String {
    std::fs::read_to_string(repo().join(AUDIT)).unwrap_or_else(|e| panic!("{AUDIT}: {e}"))
}

/// The program's own answer: every top-level command, plus the aliases that reach it.
fn product_commands() -> BTreeMap<String, Vec<String>> {
    use clap::CommandFactory;
    let mut out = BTreeMap::new();
    for sub in shall::cli::args::Cli::command().get_subcommands() {
        out.insert(
            sub.get_name().to_string(),
            sub.get_all_aliases().map(str::to_string).collect(),
        );
    }
    out
}

/// The commands a table line classifies: every backticked name in its **first** cell, because a
/// row may hold several — `lock` / `unlock` is one line and two commands, and counting rows
/// instead of commands is how a total comes to disagree with its own table.
fn commands_in_row(line: &str) -> Vec<String> {
    let Some(rest) = line.strip_prefix('|') else {
        return Vec::new();
    };
    let cell = rest.split('|').next().unwrap_or("");
    let mut out = Vec::new();
    let mut rest = cell;
    while let Some(open) = rest.find('`') {
        let after = &rest[open + 1..];
        let Some(close) = after.find('`') else { break };
        let name = after[..close].trim();
        // A cell that names a *flag* is a note about the command beside it, not a command.
        if !name.is_empty() && !name.starts_with('-') && !name.contains(' ') {
            out.push(name.to_string());
        }
        rest = &after[close + 1..];
    }
    out
}

/// Every classified command, in the order the document classifies them.
fn classified_rows() -> Vec<String> {
    audit().lines().flat_map(commands_in_row).collect()
}

/// The audit says how many commands it classified, and the division into four sections. Those
/// numbers are claims about the document, so they are checked against the document.
#[test]
fn the_stated_totals_match_the_rows() {
    let text = audit();
    let (total, output, input, read_only, exceptions) = {
        let status = text
            .lines()
            .find(|l| l.contains("top-level commands.**"))
            .expect("the audit states its total");
        let total: usize = status
            .split("**")
            .nth(1)
            .and_then(|s| s.split_whitespace().next())
            .and_then(|n| n.parse().ok())
            .expect("the total is a number");
        let count = |word: &str| -> usize {
            status
                .split(',')
                .chain(std::iter::once(status))
                .find(|p| p.contains(word))
                .and_then(|p| {
                    p.split_whitespace()
                        .find(|w| w.chars().next().is_some_and(|c| c.is_ascii_digit()))
                })
                .and_then(|n| n.trim_end_matches('.').parse().ok())
                .unwrap_or_else(|| panic!("no count beside `{word}` in: {status}"))
        };
        (
            total,
            count("output"),
            count("input"),
            count("read-only"),
            count("exception"),
        )
    };

    let sections: [(&str, usize); 4] = [
        ("### Output", output),
        ("### Input", input),
        ("### Read-only", read_only),
        ("### Named exceptions", exceptions),
    ];
    let mut seen = 0;
    for (heading, stated) in sections {
        let start = text
            .find(heading)
            .unwrap_or_else(|| panic!("the audit has no `{heading}` section"));
        let end = text[start + heading.len()..]
            .find("\n### ")
            .or_else(|| text[start + heading.len()..].find("\n## "))
            .map(|i| start + heading.len() + i)
            .unwrap_or(text.len());
        let rows = text[start..end].lines().flat_map(commands_in_row).count();
        assert_eq!(
            rows,
            stated,
            "`{heading}` states {stated} commands and lists {rows}:\n{}",
            &text[start..end]
        );
        seen += rows;
    }
    assert_eq!(
        seen, total,
        "the sections hold {seen} commands and the header states {total}"
    );
}

/// The gate `#189` asked for: the document classifies every command the program has, and every
/// row it holds is a command the program has.
#[test]
fn every_command_is_classified_exactly_once() {
    let commands = product_commands();
    let rows = classified_rows();

    let mut by_word: BTreeMap<String, usize> = BTreeMap::new();
    for row in &rows {
        *by_word.entry(row.clone()).or_default() += 1;
    }

    // A row is satisfied by a command's name or by one of its aliases.
    let mut classified: BTreeSet<String> = BTreeSet::new();
    for (name, aliases) in &commands {
        if by_word.contains_key(name) || aliases.iter().any(|a| by_word.contains_key(a)) {
            classified.insert(name.clone());
        }
    }

    let missing: Vec<&String> = commands
        .keys()
        .filter(|c| !classified.contains(*c))
        .collect();
    assert!(
        missing.is_empty(),
        "these commands are in the program and not in the audit's classification, so the model is \
         silent about what they change:\n  {}\n\
         Classify each one in {AUDIT} — reading the handler, not the name — or say in the audit why \
         it is out of scope.",
        missing
            .iter()
            .map(|c| c.as_str())
            .collect::<Vec<_>>()
            .join("\n  ")
    );

    // `help` is the one name clap synthesizes while parsing rather than holding in the command
    // tree, so it is a command a user has and not one `get_subcommands()` lists.
    const SYNTHESIZED: [&str; 1] = ["help"];
    let stale: Vec<&String> = rows
        .iter()
        .filter(|r| {
            !SYNTHESIZED.contains(&r.as_str())
                && !commands.contains_key(*r)
                && !commands.values().flatten().any(|a| a == r.as_str())
        })
        .collect();
    assert!(
        stale.is_empty(),
        "these rows name something the program does not have as a top-level command:\n  {}\n\
         A row that outlives its command is a classification of nothing.",
        stale
            .iter()
            .map(|r| r.as_str())
            .collect::<Vec<_>>()
            .join("\n  ")
    );

    let repeated: Vec<(&String, &usize)> = by_word.iter().filter(|(_, n)| **n > 1).collect();
    assert!(
        repeated.is_empty(),
        "these commands are classified more than once, so a reader cannot tell which is the \
         answer: {repeated:?}"
    );
}
