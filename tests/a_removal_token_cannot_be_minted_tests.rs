//! **Nothing outside the guard can mint a removal token.** This file replaces
//! `an_escape_hatch_names_something_that_exists_tests`, which had the same subject and is gone
//! because its subject was.
//!
//! That test watched the *reasons* given to `Reaped::for_reason`, and on 2026-08-21 it caught
//! one naming a function that did not exist — `heal`'s "each interrupted removal is enforced
//! individually in `heal_interrupted_removals`", cited by three strings and satisfied by
//! nothing. The reason string was `#[allow(dead_code)]` prose, so the check was a scan of
//! sentences. It was a real catch, and it was the only thing standing behind a claim that turned
//! out to be false in a much larger way: the constructor was public, ignored its reason, and had
//! 44 call sites, so "the token cannot be minted by a caller who would rather not ask" — the
//! guard's own words — was a convention every caller could decline.
//!
//! **The stronger gate is not a scan.** The constructor is gone: `Reaped`'s fields are private and
//! the only code that builds one is `guard.rs`. So this asks the question the old one could not:
//! *is there any way to construct one from outside that module?* A grep for the constructors, plus
//! a compile-level property stated as a doc comment on the type itself.

use std::path::{Path, PathBuf};

fn rust_sources() -> Vec<(String, String)> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().is_some_and(|e| e == "rs") {
                out.push(p);
            }
        }
    }
    let mut paths = Vec::new();
    walk(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut paths,
    );
    paths.sort();
    paths
        .into_iter()
        .filter_map(|p| {
            let body = std::fs::read_to_string(&p).ok()?;
            // **The separator is normalised here, once, and this line is not cosmetic.** The gate
            // below recognises the guard by `ends_with("src/app/sync/guard.rs")`, and on Windows
            // `Path::display()` hands back `D:\a\Shall\Shall\src\app\sync\guard.rs` — so the
            // guard failed its own exclusion and the Windows job reported the one file allowed to
            // build a removal token as an offender. Line endings were already normalised here for
            // the same class of reason, one line above.
            Some((
                p.display().to_string().replace('\\', "/"),
                body.replace("\r\n", "\n"),
            ))
        })
        .collect()
}

/// **The control for the separator, in the platform's own spelling.** This gate failed on Windows
/// because it recognised the guard by a `/`-joined suffix, so the predicate is asked directly with
/// the two spellings the same file has on the two platforms. A gate that reads a path and a gate
/// that compares one are the same gate, and only one of them had been run on both platforms.
#[test]
fn the_guard_is_recognised_by_a_path_however_the_platform_spells_it() {
    for spelled in [
        "/home/runner/work/Shall/Shall/src/app/sync/guard.rs",
        "D:\\a\\Shall\\Shall\\src\\app\\sync\\guard.rs",
    ] {
        let normalised = spelled.replace('\\', "/");
        assert!(
            normalised.ends_with("src/app/sync/guard.rs"),
            "the guard's own path went unrecognised on this platform, so the gate reported the one \
             file allowed to build a removal token as an offender: {spelled}"
        );
        assert!(
            !normalised.ends_with("src/app/sync/other.rs"),
            "the predicate accepted a path that is not the guard: {normalised}"
        );
    }
}

/// The oracle, first: a scan that has stopped matching passes by finding nothing, and this file
/// has just been the subject of one such accident.
#[test]
fn the_scan_can_see_a_constructor_it_would_miss() {
    let prod = |body: &str| constructs_a_token(&production_code_of("src/app/backends/x.rs", body));
    assert!(prod("let t = Reaped { scope, allowed: 1 };"), "a literal");
    assert!(
        prod("Reaped::for_reason(GuardScope::Sync, \"a reason\")"),
        "an associated constructor"
    );
    assert!(
        prod("pub fn reaped_for_a_unit_test() -> Reaped { Reaped { scope } }"),
        "a named constructor"
    );
    assert!(
        !prod("// Reaped::for_reason(scope, why) is what the type used to have"),
        "a mention in a comment is not a constructor"
    );
    assert!(
        !prod("fn take(r: Reaped) -> bool { r.allowed() > 0 }"),
        "using a token is not minting one"
    );
    // And the test half really is excluded, which is the other half of the rule: a `cfg(test)`
    // constructor is not compiled into a build that ships, so its call sites are not a hole.
    assert!(
        !constructs_a_token(&production_code_of(
            "src/app/shim_manager.rs",
            "fn f() {}\n#[cfg(test)]\nmod tests {\n    use super::*;\n    let r = reaped_for_a_unit_test(GuardScope::Sync);\n}\n"
        )),
        "a test module's call to the test-only constructor is not a production mint"
    );
}

/// Whether this text builds a `Reaped` outside `guard.rs`, **in code that ships**.
///
/// Three things are excluded, each for a reason that is the test's own subject rather than a
/// convenience: comments and doc comments, because a comment that *names* a constructor is
/// exactly what this repository's `S24` finding was; everything after a file's first
/// `#[cfg(test)] mod`, because the library's unit tests may call the `#[cfg(test)]` constructor
/// and that constructor is not compiled into a build anyone receives; and
/// `link_teardown_test.rs`, which is a test module `link.rs` includes rather than a test binary
/// module of its own.
fn production_code_of(path: &str, body: &str) -> String {
    let stripped: String = body
        .lines()
        .map(|l| l.split("//").next().unwrap_or(l))
        .collect::<Vec<_>>()
        .join("\n");
    if path.ends_with("link_teardown_test.rs") {
        return String::new();
    }
    // The production half ends where the first test module starts, which is a `#[cfg(test)]`
    // that opens a `mod` — not any `#[cfg(test)]`, which also appears on individual test
    // functions *inside* a module this scanner is not reading.
    let lines: Vec<&str> = stripped.lines().collect();
    let mut end = lines.len();
    for (i, line) in lines.iter().enumerate() {
        if line.trim() == "#[cfg(test)]"
            && lines
                .get(i + 1)
                .is_some_and(|next| next.trim_start().starts_with("mod "))
        {
            end = i;
            break;
        }
    }
    lines[..end].join("\n")
}

fn constructs_a_token(body: &str) -> bool {
    body.contains("Reaped {")
        || body.contains("Reaped::")
        || body.contains("reaped_for_a_unit_test(")
}

/// The gate: one file, and it is the guard.
#[test]
fn only_the_guard_builds_a_removal_token() {
    let sources = rust_sources();
    let elsewhere: Vec<String> = sources
        .iter()
        .filter(|(path, _)| !path.ends_with("src/app/sync/guard.rs"))
        .filter(|(path, body)| constructs_a_token(&production_code_of(path, body)))
        .map(|(path, _)| path.clone())
        .collect();

    assert!(
        elsewhere.is_empty(),
        "these files build a removal token, and the guard is the only thing that may:\n{}\n\n\
         A `Reaped` says the guard was asked and cleared a set. A constructor outside \
         `guard.rs` is a claim nothing backs, and the last one had 44 call sites. Tests get \
         theirs by asking the guard (`tests/harness::reaped_for_a_test`) or, for the library's own \
         unit tests, from the `#[cfg(test)]` constructor in `guard.rs` — which is not compiled \
         into a build that ships.",
        elsewhere
            .iter()
            .map(|p| format!("  {p}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}
