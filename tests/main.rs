//! **One test binary.**
//!
//! `tests/` held 101 files and cargo auto-discovers each as its own target, so every one of them
//! was fat-LTO-linked against a 100k-line crate under `codegen-units = 1` — and 36 of them never
//! call the library API at all, only spawning `CARGO_BIN_EXE_shall`. The `target/` directory that
//! produced reached **194 GB** and filled a 944 GB disk mid-build.
//!
//! **The suite already paid this cost once and wrote it down.** `mock_providers/mod.rs` opens by
//! recording that a top-level `mock_providers.rs` became *"a 716 KB binary containing zero
//! tests… compiled nineteen times"*, and that the fix was to make it a directory. This is the
//! same fix at the scale of the whole directory: one target, ninety-nine link units saved, and
//! `mock_providers` compiled once rather than nineteen times.
//!
//! **The conversion deleted nothing.** `autotests = false` in `Cargo.toml` is what stops cargo
//! claiming each file as a target, and the list below is what replaces the claim. A file added
//! to `tests/` and not listed here does not run — the one cost of this arrangement, and why
//! `every_test_file_is_in_the_suite` at the bottom fails when the two disagree.
//!
//! Four files were later renamed into the directory's filename-as-a-sentence convention, out of
//! the pre-v7 `test_*_wiring` style they were the last of; one test was deleted, by name, in
//! favour of the successor whose own header cites it as the gap it exists to close.
//!
//! Two modules carry a `cfg`: their files opened with an inner `#![cfg(...)]`, which cannot
//! survive becoming a module. On the `mod` line it is strictly better — off-platform the module
//! is not compiled at all rather than compiled to nothing.

#![allow(unused_imports)]

/// The shared test doubles. Declared once here, which is the whole point: nineteen files said
/// `mod mock_providers;` and each got its own copy.
mod mock_providers;

/// The `Fixture` sixteen files wrote out by hand, three of them differently — see the module's
/// own header for what the three ways were and why the union is the correct one.
mod harness;

/// The four assertions every scanning gate makes about its own exemption table. Nine files
/// wrote them out by hand and three of the nine were missing the one that matters.
mod ledger;

/// **The mirror.** `src/app/sync/guard.rs` owns `tests/app/sync/guard/`, and the rule is that a
/// src file's tests live in a folder of its own — so that "where is the test for this?" has an
/// answer before you have to ask it. `src/app/sync/guard.rs` was 3,473 lines, 1,944 of them
/// tests, and over the recorded ceiling because of them; the code is 1,530 lines and the ceiling
/// row is retired rather than raised.
mod app;
mod entrypoint;

/// **The mirror, continued.** `src/core/sudo.rs` owns `tests/core/sudo/`, under the same rule
/// as `src/app/sync/guard.rs`: a src file's tests live in a folder of their own, so that "where
/// is the test for this?" has an answer before you have to ask it.
mod backends;
mod config;
mod core;
mod model;
mod parsers;
mod utils;
mod verbs;

/// **The review apparatus.** The tests that read the tree rather than drive the product — the
/// gates that assert a property of the *code*, and the policies written in a workflow file
/// rather than in a function. `#120` names this folder; this is the first file to land in it.
mod review;

/// **A file in `tests/` that is not a module here never runs.**
///
/// That is the price of `autotests = false`, and it is payable only because it is checked. A
/// suite that silently stops running a file is worse than one that takes ninety-nine link units.
///
/// **The scan is recursive, because the tree is.** The mirror puts a src file's tests in a
/// folder of their own — `src/app/sync/guard.rs` owns `tests/app/sync/guard/` — and a flat scan
/// of the top level would have called that folder's contents invisible, which is the exact
/// failure this test exists to prevent, one directory down. So every `.rs` file under `tests/`
/// is checked against the module file that owns the directory it sits in, and every directory is
/// checked against the module file that owns its parent.
#[test]
fn every_test_file_is_in_the_suite() {
    fn declared_in(path: &std::path::Path) -> std::collections::BTreeSet<String> {
        std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("{} is readable: {e}", path.display()))
            .lines()
            .filter_map(|l| {
                let l = l.trim();
                let l = l
                    .strip_prefix("pub mod ")
                    .or_else(|| l.strip_prefix("mod "))?;
                l.strip_suffix(';').map(str::to_string)
            })
            .collect()
    }

    /// The file that *is* a directory's module: the `mod.rs` inside it, or a sibling file named
    /// after the directory.
    fn module_file_of(dir: &std::path::Path) -> std::path::PathBuf {
        let mod_rs = dir.join("mod.rs");
        if mod_rs.is_file() {
            return mod_rs;
        }
        let name = dir
            .file_name()
            .expect("a directory has a name")
            .to_string_lossy();
        let sibling = dir.with_file_name(format!("{name}.rs"));
        if sibling.is_file() {
            return sibling;
        }
        mod_rs
    }

    /// **A directory with no Rust in it is data, not a module.** `tests/fixtures/` holds 47
    /// captured command outputs and no source, and requiring a `mod` declaration for it would be
    /// a declaration of nothing. The test is the other way round: a directory that *does* hold
    /// Rust is a module, and its files are its subject.
    fn holds_rust(dir: &std::path::Path) -> bool {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return false;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|e| e == "rs") {
                return true;
            }
            if path.is_dir() && holds_rust(&path) {
                return true;
            }
        }
        false
    }

    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let mut missing: Vec<String> = Vec::new();

    // The root is `main.rs`, and it owns everything directly in `tests/`.
    let root_declared = declared_in(&root.join("main.rs"));
    for entry in std::fs::read_dir(&root)
        .expect("tests/ is readable")
        .flatten()
    {
        let path = entry.path();
        let name = path
            .file_name()
            .expect("an entry has a name")
            .to_string_lossy()
            .into_owned();
        if name == "main.rs" {
            continue;
        }
        if path.is_file() {
            let stem = name.strip_suffix(".rs").expect("a .rs file has a stem");
            if !root_declared.contains(stem) {
                missing.push(format!("tests/{name} is not a module of the suite"));
            }
        } else if path.is_dir() {
            // A directory is a module of its parent — unless it holds no Rust at all, which
            // makes it data.
            if holds_rust(&path) && !root_declared.contains(&name) {
                missing.push(format!("tests/{name}/ is not a module of the suite"));
            }
        }
    }

    // Every directory below the root, and every file in it.
    let mut dirs = vec![root.clone()];
    while let Some(dir) = dirs.pop() {
        // The root's own module file is the suite's root, `main.rs`; every other directory's is
        // its `mod.rs` or the file named after it.
        let owner = if dir == root {
            dir.join("main.rs")
        } else {
            module_file_of(&dir)
        };
        let declared = if owner.is_file() {
            declared_in(&owner)
        } else {
            std::collections::BTreeSet::new()
        };
        for entry in std::fs::read_dir(&dir)
            .expect("a directory is readable")
            .flatten()
        {
            let path = entry.path();
            let name = path
                .file_name()
                .expect("an entry has a name")
                .to_string_lossy()
                .into_owned();
            if path.is_dir() {
                if holds_rust(&path) && !declared.contains(&name) {
                    missing.push(format!(
                        "{} is not a module of {}",
                        path.strip_prefix(&root).expect("under tests/").display(),
                        owner.strip_prefix(&root).expect("under tests/").display()
                    ));
                }
                dirs.push(path);
            } else if path.is_file() && name.ends_with(".rs") {
                let stem = name.strip_suffix(".rs").expect("a .rs file has a stem");
                if owner.file_name().map(|n| n == "mod.rs").unwrap_or(false)
                    || name == "mod.rs"
                    || (dir == root && name == "main.rs")
                {
                    // The directory's own module file.
                    continue;
                }
                if !declared.contains(stem) {
                    missing.push(format!(
                        "{} is not a module of {}",
                        path.strip_prefix(&root).expect("under tests/").display(),
                        owner.strip_prefix(&root).expect("under tests/").display()
                    ));
                }
            }
        }
    }

    assert!(
        missing.is_empty(),
        "these files are in tests/ and are not modules of the suite, so nothing runs them: \
         {missing:#?}\nAdd `mod <name>;` to the module file that owns them."
    );

    // And the other direction: a `mod` naming a file that is gone does not compile, so the only
    // thing left to check is that the scan found the suite at all.
    assert!(
        root_declared.len() >= 10,
        "only {} module(s) found in tests/main.rs; this scan has stopped matching it",
        root_declared.len()
    );
}

mod cli;

mod e2e;

mod integration;
