//! A fixture that redirects one root and not the others is not hermetic (`#158`).
//!
//! **What it cost.** `tests/e2e/dry_run_tests.rs` set `SHALL_CONFIG_DIR`/`SHALL_DATA_DIR` and ran
//! `schedule add nightly` for real, as the control half of a preview test. A `schedule:` is
//! provisioned onto the *OS* scheduler, and the unit directory is `dirs::config_dir()` —
//! `$XDG_CONFIG_HOME`, else `$HOME`, and the `FOLDERID`/`Library` directories on the other two
//! platforms — none of which a fixture's own root moves. So every run of the suite installed a
//! real `shall-nightly.service` in the operator's `~/.config/systemd/user`, with the double
//! `ExecStart` of `#158`, and the owner's only notice of it was a desktop notification a week
//! later, when the timer fired and failed.
//!
//! **Why this is a gate and not a fix.** Thirty fixtures spawn the binary with a config root and
//! each one was written by a different session; the shape of the mistake is identical in all of
//! them, so the cheap thing to fix was one file and the cheap thing to *keep* fixed is the rule.
//! A fixture that reaches the binary with a `SHALL_CONFIG_DIR` must also name a `HOME`, or the
//! next one repeats the week.

use std::path::Path;

/// Every `.rs` under `tests/`, so a new file is covered the day it lands rather than when someone
/// remembers to add it to a list.
fn test_sources(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            test_sources(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

#[test]
fn a_fixture_that_redirects_the_config_root_also_isolates_the_home() {
    let tests = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let mut sources = Vec::new();
    test_sources(&tests, &mut sources);
    assert!(
        sources.len() > 50,
        "the scan found {} files under {}, so it is not looking where the fixtures are",
        sources.len(),
        tests.display()
    );

    let mut offenders = Vec::new();
    for path in &sources {
        // This file names the variable in its own prose and spawns nothing.
        if path.file_name().and_then(|n| n.to_str())
            == Some("a_fixture_that_redirects_one_root_redirects_them_all.rs")
        {
            continue;
        }
        let Ok(source) = std::fs::read_to_string(path) else {
            continue;
        };
        if !source.contains("CARGO_BIN_EXE_shall") || !source.contains("SHALL_CONFIG_DIR") {
            continue;
        }
        // `env_remove` is a fixture that carries no root at all: it is testing the default, and
        // there is nothing for it to redirect.
        if !source.contains(".env(\"SHALL_CONFIG_DIR\"") {
            continue;
        }
        if !source.contains(".env(\"HOME\"") {
            offenders.push(
                path.strip_prefix(&tests)
                    .unwrap_or(path)
                    .display()
                    .to_string(),
            );
        }
    }

    assert!(
        offenders.is_empty(),
        "these fixtures spawn the binary with a config root and leave HOME alone, so anything \
         they provision lands in the operator's own home — a systemd unit, a launchd agent, a \
         Task Scheduler entry — and outlives the test:\n  {}\n\nAdd `.env(\"HOME\", <the fixture \
         dir>)` (and `XDG_CONFIG_HOME`/`XDG_DATA_HOME`, which win over `HOME` when set) beside \
         the `SHALL_CONFIG_DIR` line.",
        offenders.join("\n  ")
    );
}
