use crate::core::Package;
use crate::parsers::{or_unrecognised, OutputParser, ParseResult};
use crate::utils::text::sanitize;

/// apt's parser. A dedicated struct rather than a `LambdaParser` because apt is the one
/// backend that can also report which packages dpkg itself refuses to lose.
pub struct AptParser;

impl OutputParser for AptParser {
    fn parse_installed(&self, output: &str) -> ParseResult {
        parse_list(output)
    }
    fn parse_search(&self, output: &str) -> Vec<Package> {
        parse_search(output)
    }
    fn parse_essential(&self, output: &str) -> Vec<String> {
        parse_essential(output)
    }
}

/// Parses `dpkg-query -W -f='${Essential} ${Priority} ${Package}\n'`, keeping only the
/// packages Debian marks as undeletable.
///
/// `Essential: yes` is enforced by dpkg itself ("the package management system will refuse
/// to remove the package"), and `Priority: required` means "necessary for the proper
/// functioning of the system". Both are read live from the running system, so this stays
/// correct across distro releases without a hand-maintained name list.
///
/// Expected input format: "yes required base-files" / "no optional python3".
///
/// The name is read from the END and the flags from the front, deliberately: `Priority` is
/// an optional dpkg field, and a package that omits it yields two tokens rather than
/// three. Counting positions from the front would then read the *name* as the priority and
/// drop the package — silently un-protecting something marked `Essential: yes`. This query
/// exists to keep systems alive, so it must fail closed.
pub fn parse_essential(output: &str) -> Vec<String> {
    sanitize(output)
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            let name = parts.last()?;
            let flags = &parts[..parts.len() - 1];
            (flags.contains(&"yes") || flags.contains(&"required")).then(|| name.to_string())
        })
        .collect()
}

/// Every status word dpkg can put on a package, and whether it means the software is on the
/// machine.
///
/// The list is exhaustive on purpose: it is what tells a real row from a line of apt's error
/// output, and a status dpkg grows tomorrow should be an unreadable line rather than a silent
/// "absent".
const DPKG_STATUS_WORDS: [(&str, bool); 8] = [
    ("installed", true),
    // Installed, with a trigger deferred — not a partial install. Reading these as absent
    // would report a working package as missing and make `sync` reinstall it, which is B0's
    // mistake pointed the other way.
    ("triggers-awaited", true),
    ("triggers-pending", true),
    // The software is not usable. `config-files` is what `apt remove` leaves behind.
    ("not-installed", false),
    ("config-files", false),
    ("half-installed", false),
    ("unpacked", false),
    ("half-configured", false),
];

/// One row of `dpkg-query -W -f='${db:Status-Status} ${Package} ${Version}\n'`, as
/// `(is_installed, name, version)`. `None` is a line this format cannot have produced.
///
/// A removed package's `${Version}` is empty, so the version is optional and its absence is
/// not a parse failure.
fn read_row(line: &str) -> Option<(bool, &str, &str)> {
    let mut parts = line.split_whitespace();
    let status = parts.next()?;
    let installed = DPKG_STATUS_WORDS
        .iter()
        .find_map(|(word, present)| (*word == status).then_some(*present))?;
    let name = parts.next()?;
    Some((installed, name, parts.next().unwrap_or("")))
}

/// Parses output from the debian/ubuntu package database query.
/// Command: dpkg-query -W -f='${db:Status-Status} ${Package} ${Version}\n'
/// Expected input format: "installed curl 7.81.0-1ubuntu1.16"
///
/// **The status field is what this argv exists for.** `dpkg-query -W` alone lists every package
/// dpkg knows about, and dpkg keeps knowing about one after `apt remove`: it enters
/// `deinstall ok config-files`, meaning the software is gone and its configuration was kept.
/// Shall removes with `remove` and not `purge`, which is the correct and safe choice — so every
/// conffile-carrying package Shall removed was minted into a state the old listing reported as
/// installed. `list` named a package that was not there, `check` saw no drift, and `sync`
/// refused to put it back, permanently (B0). The lister was the bug, not the remover.
///
/// **It also settles what a line that is not a package looks like.** apt's own error output
/// (`E: Could not open lock file`) used to read as a package named `E:` at version *"Could not
/// open lock file"*, because any line with a space in it was a package. A row now has to open
/// with a status word dpkg can actually emit, so junk is unreadable rather than believed.
pub fn parse_list(output: &str) -> ParseResult {
    let clean = sanitize(output);
    let candidates = crate::parsers::data_lines(&clean);
    let found = candidates
        .iter()
        .filter_map(|line| {
            let (installed, name, ver) = read_row(line)?;
            installed.then(|| match ver.is_empty() {
                // An installed package always carries a version, but `Some("")` compares as a
                // version and would make every plan see a change it cannot explain.
                true => Package::new(name, "apt"),
                false => Package::with_version(name, ver, "apt"),
            })
        })
        .collect();
    // Only the lines this format could not have produced count towards "the parser did not
    // understand this". A row read correctly and dropped for saying `config-files` is an
    // answer, and a machine whose every known package has been removed is a real machine —
    // reporting that as unrecognised would refuse to run on it.
    let unreadable: Vec<&str> = candidates
        .iter()
        .copied()
        .filter(|line| read_row(line).is_none())
        .collect();
    or_unrecognised("apt", found, &unreadable)
}

/// Parses the output of 'apt-cache search' for remote package discovery.
/// Expected input format: "package-name - human readable description"
pub fn parse_search(output: &str) -> Vec<Package> {
    sanitize(output)
        .lines()
        .filter_map(|line| {
            // Apt-cache search usually uses " - " as a separator between name and description
            let (name, desc) = line.split_once(" - ")?;
            let mut p = Package::new(name.trim(), "apt");
            p.properties
                .insert("description".to_string(), desc.trim().to_string());
            Some(p)
        })
        .collect()
}

/// `apt list --upgradable` (`Q44`).
///
/// ```text
/// Listing...
/// gzip/noble-updates,noble-security 1.12-1ubuntu3.2 amd64 [upgradable from: 1.12-1ubuntu3.1]
/// ```
///
/// The name is what precedes the `/`; everything after it is the suite list, not part of the
/// name. The version taken is the *first* one — the second, inside the brackets, is what is
/// installed, and reading that one reports every package as already current.
pub fn parse_apt_outdated(output: &str) -> Vec<Package> {
    crate::utils::text::sanitize(output)
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            // apt's own progress banner, and its warning about an unstable CLI.
            if line.is_empty() || line.starts_with("Listing") || line.starts_with("WARNING") {
                return None;
            }
            let (name, rest) = line.split_once('/')?;
            let version = rest.split_whitespace().nth(1)?;
            if name.is_empty() || version.is_empty() {
                return None;
            }
            Some(Package::with_version(name.trim(), version, "apt"))
        })
        .collect()
}
