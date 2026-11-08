//! Parsers for the BSD package tools: FreeBSD's `pkg` and OpenBSD's `pkg_add`/`pkg_info` (U26).
//!
//! Both list installed packages as `name-version   description`, exactly the pkgsrc shape, and
//! the name itself can contain dashes (`py311-requests-2.31.0`) — so the split point is the LAST
//! dash whose right-hand side starts with a digit. This is the same heuristic `pkgsrc::parse_pkgin`
//! uses; kept in its own module because the backend labels differ and a shared label would put
//! the wrong prefix on the packages.

use crate::core::Package;
use crate::parsers::{or_unrecognised, ParseResult};
use crate::utils::text::sanitize;

/// Split a `name-version` token into `(name, version)`. `None` when no dash is followed by a
/// digit (a line with no version, e.g. a legend line).
fn split_name_version(token: &str) -> Option<(&str, &str)> {
    let bytes = token.as_bytes();
    for (idx, _) in token.rmatch_indices('-') {
        if let Some(&next) = bytes.get(idx + 1) {
            if next.is_ascii_digit() {
                return Some((&token[..idx], &token[idx + 1..]));
            }
        }
    }
    None
}

pub(crate) fn parse_with_backend(output: &str, backend: &str) -> ParseResult {
    let clean = sanitize(output);
    let candidates = crate::parsers::data_lines(&clean);
    let found = candidates
        .iter()
        .filter_map(|line| {
            let token = line.split_whitespace().next()?;
            match split_name_version(token) {
                Some((name, version)) if !name.is_empty() => {
                    Some(Package::with_version(name, version, backend))
                }
                _ => None,
            }
        })
        .collect();
    or_unrecognised(backend, found, &candidates)
}

/// FreeBSD: `pkg info` (installed) and `pkg search` (remote catalogue) share the layout.
pub fn parse_pkg(output: &str) -> ParseResult {
    parse_with_backend(output, "pkg")
}

/// OpenBSD: `pkg_info` (installed) and `pkg_info -Q` (query the remote) share the layout.
pub fn parse_pkg_add(output: &str) -> ParseResult {
    parse_with_backend(output, "pkg_add")
}

// ---------------------------------------------------------------------------
// Void Linux (`xbps`). Here rather than in its own module because the split point is the same
// question `split_name_version` above already answers — a `pkgver` token whose version starts
// after the last dash — and a second copy of that rule is how `pkgsrc.rs` came to be `bsd.rs`
// byte-for-byte.
// ---------------------------------------------------------------------------

/// Split an XBPS `pkgver` token (`<name>-<version>_<revision>`, e.g. `bash-5.2.15_2`) into
/// `(name, version)`. The version always begins with a digit after the final `-`, which is what
/// separates it from names that themselves contain dashes (`xbps-triggers-0.128_1`).
fn split_pkgver(tok: &str) -> Option<(&str, &str)> {
    let (name, ver) = tok.rsplit_once('-')?;
    if ver.chars().next()?.is_ascii_digit() {
        Some((name, ver))
    } else {
        None
    }
}

/// Every state word `xbps-query -l` prints, and whether it means the software is installed.
///
/// `lib/transaction_commit.c` unpacks a package, **registers it as `XBPS_PKG_STATE_UNPACKED`**
/// and only then runs its configure half — so a machine killed in between comes back with the
/// package in the database and not usable, which is `uu`. `hr` is a removal that stopped
/// part-way and `??` is a state this xbps does not know; neither is software on the machine.
///
/// Exhaustive on purpose, for the reason `parsers/apt.rs`'s table is: a state xbps grows
/// tomorrow should be an unreadable line rather than a silent "installed".
const XBPS_STATE_WORDS: [(&str, bool); 4] =
    [("ii", true), ("uu", false), ("hr", false), ("??", false)];

/// One row of either shape this listing comes in: `(is_installed, pkgver token)`.
///
/// `-l` prints `<state> <pkgver> <desc>`; `-m` prints the bare pkgver, and every row there is
/// manually installed by construction. `None` is a line neither shape could have produced.
fn read_xbps_row(line: &str) -> Option<(bool, &str)> {
    let mut tokens = line.split_whitespace();
    let first = tokens.next()?;
    match XBPS_STATE_WORDS
        .iter()
        .find_map(|(word, installed)| (*word == first).then_some(*installed))
    {
        Some(installed) => Some((installed, tokens.next()?)),
        None => Some((true, first)),
    }
}

/// `xbps-query -l` / `xbps-query -m`. `-l` lines carry a two-character state flag
/// (`ii <pkgver> <desc>`); `-m` lines are the bare pkgver.
///
/// **The flag is load-bearing, and reading past it was the crash section's defect.** `-l`
/// lists the package *database*, not the installed set: a package that is registered and
/// unpacked, or half-removed, prints a row exactly like a working one. Taking the first token
/// that parsed as a pkgver made every one of them "installed", so `sync` planned nothing for a
/// declared package the machine did not have and reported convergence over it.
pub fn parse_xbps_list(output: &str) -> ParseResult {
    let clean = sanitize(output);
    let candidates = crate::parsers::data_lines(&clean);
    let found = candidates
        .iter()
        .filter_map(|line| {
            let (installed, token) = read_xbps_row(line)?;
            let (name, ver) = split_pkgver(token)?;
            installed.then(|| Package::with_version(name, ver, "xbps"))
        })
        .collect();
    // A row read correctly and dropped for its state is an answer, not a shape this file could
    // not read — a machine whose every registered package is half-installed is a real machine,
    // and refusing the listing would turn it into "unknown" rather than "nothing usable here".
    let unreadable: Vec<&str> = candidates
        .iter()
        .copied()
        .filter(|line| read_xbps_row(line).is_none_or(|(_, token)| split_pkgver(token).is_none()))
        .collect();
    or_unrecognised("xbps", found, &unreadable)
}

/// `xbps-query -Rs <query>`: `[-] bash-5.2.15_2   The GNU Bourne Again Shell`, where `[*]`
/// means installed.
pub fn parse_xbps_search(output: &str) -> Vec<Package> {
    let mut packages = Vec::new();
    for line in sanitize(output).lines() {
        let tokens: Vec<&str> = line.split_whitespace().collect();
        let Some(pos) = tokens.iter().position(|t| split_pkgver(t).is_some()) else {
            continue;
        };
        let Some((name, ver)) = split_pkgver(tokens[pos]) else {
            continue;
        };
        let mut pkg = Package::with_version(name, ver, "xbps");
        let desc = tokens[pos + 1..].join(" ");
        if !desc.is_empty() {
            pkg.properties.insert("description".to_string(), desc);
        }
        packages.push(pkg);
    }
    packages
}

/// `xbps-query -x <pkg>` — run-time dependencies as version-constrained patterns
/// (`glibc>=2.36_1`), one per line. The constraint is not part of the name a later command
/// would have to use.
pub fn parse_xbps_dependencies(output: &str) -> Vec<String> {
    sanitize(output)
        .lines()
        .filter_map(|l| l.split(['>', '<', '=']).next())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}
