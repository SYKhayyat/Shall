use crate::core::Package;
use crate::parsers::{or_unrecognised, ParseResult};
use crate::utils::text::sanitize;

/// Parses output from 'pacman -Q' for installed packages.
/// Expected input format: "name version"
pub fn parse_list(output: &str) -> ParseResult {
    parse_list_for(output, "pacman")
}

/// Like [`parse_list`], but stamps a caller-supplied backend name. AUR helpers
/// (`yay`, `paru`) speak pacman's `-Q` syntax verbatim but must label their packages
/// with their own backend so state tracking stays per-backend correct.
pub fn parse_list_for(output: &str, backend: &str) -> ParseResult {
    let clean = sanitize(output);
    let candidates = crate::parsers::data_lines(&clean);
    let found = candidates
        .iter()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let name = parts.next()?;
            let ver = parts.next()?;
            Some(Package::with_version(name, ver, backend))
        })
        .collect();
    or_unrecognised(backend, found, &candidates)
}

/// Parses the multi-line output of 'pacman -Ss' for remote searching.
/// Pacman search output typically has the Name/Version on one line and the Description on the next.
pub fn parse_search(output: &str) -> Vec<Package> {
    parse_search_for(output, "pacman")
}

/// Like [`parse_search`], but stamps a caller-supplied backend name (see [`parse_list_for`]).
pub fn parse_search_for(output: &str, backend: &str) -> Vec<Package> {
    let clean = sanitize(output);
    let mut packages = Vec::new();
    let mut lines = clean.lines().peekable();

    while let Some(line) = lines.next() {
        // An indented line is a description continuation, not a package.
        if line.starts_with(' ') || line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        // Format is usually: core/bash 5.1.016-1 (base) [installed]
        if let Some(repo_name) = parts.first() {
            let name = repo_name.split('/').next_back().unwrap_or(repo_name);
            let mut p = Package::new(name, backend);

            if let Some(version) = parts.get(1) {
                p.version = Some(version.to_string());
            }

            if let Some(desc_line) = lines.peek() {
                if desc_line.starts_with("    ") {
                    p.properties
                        .insert("description".to_string(), desc_line.trim().to_string());
                    lines.next(); // Consume the description line
                }
            }
            packages.push(p);
        }
    }
    packages
}

/// The `Depends On` row of a `pacman -Si` report, as package names.
///
/// **Matched by key, not by column.** This stripped the literal `"Depends On     :"` — five
/// spaces — and Arch prints six, so the prefix never matched and `pacman:` answered every
/// dependency query with an empty list for the whole life of the backend. Nothing caught it,
/// because the only consumer was a planner that installed whatever came back and was better off
/// with nothing (`Y9`); now that the answer is only ever *shown*, an empty one is a lie a user
/// reads. Captured from `pacman -Si jq` in the Arch integration image, 2026-08-06:
///
/// ```text
/// Depends On      : glibc  oniguruma
/// ```
///
/// `Optional Deps` and `Conflicts With` are rows of the same shape, which is why the key is
/// compared whole rather than by prefix. And the row carries SEVERAL names, which is why the
/// labelled parser in `generic.rs` — written for apt's one-name-per-line shape — cannot read it.
pub fn parse_depends_on(output: &str) -> Vec<String> {
    let mut deps = Vec::new();
    for line in crate::utils::text::sanitize(output).lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        if key.trim() != "Depends On" {
            continue;
        }
        for part in value.split_whitespace() {
            // pacman writes `None` where there are no dependencies at all.
            if part == "None" {
                continue;
            }
            // `glibc>=2.36` and `sh=5` are one dependency each; the constraint is not part of
            // the name a later command would have to use.
            let name = part.split(['>', '<', '=']).next().unwrap_or(part);
            if !name.is_empty() {
                deps.push(name.to_string());
            }
        }
    }
    deps
}

/// `pacman -Qu` (`Q44`): `name installed -> available`, one per line.
pub fn parse_pacman_outdated(output: &str) -> Vec<Package> {
    crate::utils::text::sanitize(output)
        .lines()
        .filter_map(|line| {
            let (head, available) = line.split_once("->")?;
            let name = head.split_whitespace().next()?;
            let available = available.trim();
            if name.is_empty() || available.is_empty() {
                return None;
            }
            Some(Package::with_version(name, available, "pacman"))
        })
        .collect()
}
