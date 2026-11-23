//! A fixture that redirects one root and not the others is not hermetic (`#158`, `#219`).
//!
//! **What it cost, the first time.** `tests/e2e/dry_run_tests.rs` set `SHALL_CONFIG_DIR`/
//! `SHALL_DATA_DIR` and ran `schedule add nightly` for real, as the control half of a preview
//! test. A `schedule:` is provisioned onto the *OS* scheduler, and the unit directory is
//! `dirs::config_dir()` — `$XDG_CONFIG_HOME`, else `$HOME`, and the `FOLDERID`/`Library`
//! directories on the other two platforms — none of which a fixture's own root moves. So every
//! run of the suite installed a real `shall-nightly.service` in the operator's
//! `~/.config/systemd/user`, with the double `ExecStart` of `#158`, and the owner's only notice
//! of it was a desktop notification a week later, when the timer fired and failed.
//!
//! **What it cost, the second time.** `#219`: a hand run with `HOME=` set to a scratch directory
//! still wrote the real `~/.config/shall/schedules` and overwrote `modules/adopted.txt`, because
//! `$XDG_CONFIG_HOME` was exported and it outranks `$HOME/.config`. `HOME` is the variable
//! everybody reaches for and it is not a lever on a desktop that sets the XDG ones. The harness
//! had recorded the same trap twice already (`$XDG_CONFIG_HOME` into a real
//! `~/.config/systemd/user`, 2026-11-02, landing `#130`; `npm` reading a real `~/.npmrc`,
//! 2026-11-08), which is the measure of how much a rule written in a comment is worth.
//!
//! **Why this is a gate and not a fix.** Thirty fixtures spawn the binary with a config root and
//! each one was written by a different session; the shape of the mistake is identical in all of
//! them, so the cheap thing to fix was one file and the cheap thing to *keep* fixed is the rule.
//! A test that reaches the binary must redirect all four levers — `SHALL_CONFIG_DIR`,
//! `SHALL_DATA_DIR`, `HOME`, and `XDG_CONFIG_HOME` — or appear in the table below with a reason.
//! `--config-dir` alone is not enough either: it moves what you declared while the run plans a
//! fresh sandbox against the real machine's managed state, which is the state the removal guard
//! consults before it deletes anything.
//!
//! The scan reads code, not comments, so the files that merely *name* these variables in prose
//! are not reported. That is deliberate: a comment is not a caller.

use std::path::{Path, PathBuf};

/// Files that reach the binary without redirecting every root, and why that is safe.
///
/// Each row is a claim, so each is checkable — `every_exception_still_describes_something`
/// fails when one stops being true, which is what keeps an exception table from becoming a list
/// of permissions nobody re-read.
const EXCEPTIONS: &[(&str, &str)] = &[
    (
        "tests/e2e/help_map_tests.rs",
        "runs `--help` only: clap prints its own command list and opens no config",
    ),
    (
        "tests/e2e/a_sweep_that_must_not_delete_another_configs_schedule_tests.rs",
        "uses `CARGO_BIN_EXE_shall` as the path a systemd unit would exec; it never spawns it",
    ),
    (
        "tests/review/a_fixture_that_redirects_one_root_redirects_them_all.rs",
        "this gate: it searches for the literal `CARGO_BIN_EXE_shall` and spawns nothing",
    ),
];

/// Every `.rs` under `tests/`, so a new file is covered the day it lands rather than when someone
/// remembers to add it to a list.
fn test_sources(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            test_sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// The file's code, with every line comment removed.
///
/// `//!` and `///` count as comments: this file's own header names the variables, and so does
/// `named_commands_exist_tests`' — a doc link to a variable is not a use of it.
fn code_of(body: &str) -> String {
    body.lines()
        .map(|line| match line.find("//") {
            Some(at) => &line[..at],
            None => line,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn relative(path: &Path) -> String {
    path.strip_prefix(PathBuf::from(env!("CARGO_MANIFEST_DIR")))
        .unwrap_or(path)
        .components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

/// The four levers, and what each one moves.
const LEVERS: &[(&str, &str)] = &[
    ("SHALL_CONFIG_DIR", "the config repo"),
    ("SHALL_DATA_DIR", "the registry, journal and snapshots"),
    (
        "HOME",
        "`$HOME/.config` and every manager that keeps state there",
    ),
    (
        "XDG_CONFIG_HOME",
        "`dirs::config_dir()` — it wins over `HOME`, which is the whole of `#219`",
    ),
];

#[test]
fn a_test_that_runs_the_binary_redirects_every_root() {
    let tests = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let mut sources = Vec::new();
    test_sources(&tests, &mut sources);
    assert!(
        sources.len() > 50,
        "the scan found {} files under {}, so it is not looking where the fixtures are",
        sources.len(),
        tests.display()
    );

    let mut spawners = 0usize;
    let mut offenders: Vec<String> = Vec::new();
    for path in &sources {
        let Ok(source) = std::fs::read_to_string(path) else {
            continue;
        };
        let code = code_of(&source);
        if !code.contains("CARGO_BIN_EXE_shall") {
            continue;
        }
        spawners += 1;
        let name = relative(path);
        if EXCEPTIONS.iter().any(|(p, _)| *p == name) {
            continue;
        }
        let missing: Vec<&str> = LEVERS
            .iter()
            .filter(|(needle, _)| !code.contains(needle))
            .map(|(_, moves)| *moves)
            .collect();
        if !missing.is_empty() {
            offenders.push(format!("{name} leaves alive: {}", missing.join(", ")));
        }
    }

    assert!(
        spawners > 20,
        "only {spawners} file(s) were seen to spawn the binary — the scan has stopped matching, \
         so a pass here would mean nothing"
    );
    assert!(
        offenders.is_empty(),
        "these tests spawn the real binary and leave one of its roots pointing at the machine \
         they run on, so anything they provision — a systemd unit, a launchd agent, a Task \
         Scheduler entry, a package — outlives the test and lands on the operator:\n  {}\n\n\
         Set all four, as the `Fixture` harness does, or add the file to EXCEPTIONS with a \
         reason. `--config-dir` alone is not isolation: without `--data-dir` the run plans \
         against the real machine's managed state.",
        offenders.join("\n  ")
    );
}

/// The exception table is a claim per row: a file that no longer reaches the binary is a stale
/// excuse, and this is what makes the table auditable in both directions.
#[test]
fn every_exception_still_describes_something() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut stale: Vec<&str> = Vec::new();
    for (path, _) in EXCEPTIONS {
        match std::fs::read_to_string(root.join(path)) {
            Ok(body) if code_of(&body).contains("CARGO_BIN_EXE_shall") => {}
            _ => stale.push(path),
        }
    }
    assert!(
        stale.is_empty(),
        "these exception rows no longer describe anything: {stale:?}"
    );
}
