//! **The two release scripts must gate the same things** (#196).
//!
//! `scripts/release-check.sh` and `scripts/release-check.ps1` are the same question asked on two
//! platforms, and `PLAN.md`'s M1 box requires both green before a release. They had drifted:
//! `decision-count.sh --check` is a HARD gate on Linux and was **not run at all** on Windows, so
//! the Windows script could print `=====> GO` while a documented count disagreed with the decision
//! register — the one thing that check exists to catch, on the platform nobody was watching.
//!
//! The drift is structural, not clerical: two files, two languages, one gate list, and nothing
//! comparing them. A person adding a gate to the shell script has no reason to open the PowerShell
//! one, and the PowerShell one has no way to notice. So the comparison lives here: every gate the
//! shell script runs, the PowerShell script runs too.
//!
//! **What this cannot see.** It compares invocation keys, not what a gate measures — the same
//! limitation `grader_gate_parity_tests` names at length. A gate that runs with a weaker argument on
//! one platform passes this. It is a floor, and the floor is what was missing.

use std::path::PathBuf;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(repo().join(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

/// Every gate the shell script runs, as the token that names it: a `scripts/*.sh` path or a
/// `cargo <subcommand>`. Comments are skipped — a gate named only in prose is not a gate.
fn gates_in_shell() -> Vec<String> {
    let text = read("scripts/release-check.sh");
    let mut out = Vec::new();
    for line in text.lines() {
        let code = line.trim();
        if code.starts_with('#') {
            continue;
        }
        // `scripts/foo.sh` anywhere in a live line.
        let mut rest = code;
        while let Some(i) = rest.find("scripts/") {
            let tail = &rest[i..];
            let end = tail
                .find(|c: char| c.is_whitespace() || c == '"' || c == '\'')
                .unwrap_or(tail.len());
            let name = &tail[..end];
            if name.ends_with(".sh") && !out.iter().any(|g| g == name) {
                out.push(name.to_string());
            }
            rest = &tail[end..];
        }
        // `cargo <subcommand>` — the subcommand is what the gate is.
        for sub in [
            "fmt", "clippy", "test", "build", "mutants", "deny", "check", "nextest",
        ] {
            let needle = format!("cargo {sub}");
            let mut probe = code;
            while let Some(i) = probe.find(&needle) {
                let after = probe[i + needle.len()..].chars().next();
                // `cargo test` but not `cargo testing`.
                let key = format!("cargo {sub}");
                if !after.is_some_and(|c| c.is_alphanumeric() || c == '-' || c == '_')
                    && !out.iter().any(|g| g == &key)
                {
                    out.push(key);
                }
                probe = &probe[i + needle.len()..];
            }
        }
    }
    out
}

/// The same normalisation on the PowerShell side: quotes dropped, whitespace collapsed, so
/// `& $bash "scripts/x.sh" "--check"` and `sh scripts/x.sh --check` compare equal.
/// Comment lines are excluded — a gate named only in the header is a *name*, not a gate, which
/// is the exact failure `grader_gate_parity_tests` names: the header said what a reader should
/// expect and the header was wrong.
fn flattened(text: &str) -> String {
    let code: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();
    code.join(" ")
        .replace(['"', '\''], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn every_gate_the_shell_script_runs_is_a_gate_the_powershell_script_runs() {
    let ps1 = flattened(&read("scripts/release-check.ps1"));
    let gates = gates_in_shell();
    assert!(
        gates.len() > 5,
        "the derivation found {} gates, so it is not reading the script: {gates:?}",
        gates.len()
    );

    let missing: Vec<String> = gates
        .iter()
        .filter(|g| {
            if ps1.contains(g.as_str()) {
                return false;
            }
            // `install.sh` gates the Linux side and is a deliberate pair with `install.ps1`:
            // each release script runs its own platform's installer for real. The twin is
            // accepted only when the `.ps1` side actually names it.
            if let Some(stem) = g.strip_suffix(".sh") {
                let twin = format!("{stem}.ps1");
                return !flattened(&read("scripts/release-check.ps1")).contains(&twin);
            }
            true
        })
        .cloned()
        .collect::<Vec<_>>();
    assert!(
        missing.is_empty(),
        "these gates run on Linux and not on Windows, so `release-check.ps1` can print GO on a \
         tree `release-check.sh` refuses:\n  {}\n\
         Add them to scripts/release-check.ps1 (it finds Git-Bash with Find-Bash, so a POSIX \
         script is reachable) or delete them from both.",
        missing.join("\n  ")
    );
}

/// The gate that was missing, named, so the walk above cannot lose it in a refactor: a *counted*
/// document check is only a gate if it runs on both.
#[test]
fn the_decision_register_is_checked_on_both_platforms() {
    for (rel, script) in [
        ("scripts/release-check.sh", "shell"),
        ("scripts/release-check.ps1", "PowerShell"),
    ] {
        let flat = flattened(&read(rel));
        assert!(
            flat.contains("scripts/decision-count.sh --check"),
            "the {script} release script does not check the decision register, so it can report \
             GO while a documented count disagrees with it"
        );
    }
}

/// The Windows script's own hard gates are listed in its header, and a gate that runs without
/// being listed is what made this defect invisible: the list said what a reader should expect and
/// the list was wrong.
#[test]
fn the_powershell_header_lists_the_decision_register() {
    let text = read("scripts/release-check.ps1");
    let header: String = text
        .lines()
        .take_while(|l| l.starts_with('#') || l.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        header.contains("decision-count.sh"),
        "the header does not say the decision register is checked:\n{header}"
    );
}
