use crate::core::Package;
use crate::parsers::{or_unrecognised, ParseResult};
use crate::utils::text::sanitize;

/// Standard RPM query parser used by DNF and Zypper.
/// Command: rpm -qa --queryformat '%{NAME}|%{VERSION}\n'
/// Expected input format: "package-name|1.2.3-r1"
pub fn parse_rpm_qa(output: &str, backend: &str) -> ParseResult {
    let clean = sanitize(output);
    let candidates = crate::parsers::data_lines(&clean);
    let found = candidates
        .iter()
        .filter_map(|l| {
            let (name, ver) = l.split_once('|')?;
            Some(Package::with_version(name.trim(), ver.trim(), backend))
        })
        .collect();
    or_unrecognised(backend, found, &candidates)
}

/// The architectures dnf appends to a name. Only these are stripped: taking everything
/// before the first `.` turned `python3.12.x86_64` into `python3`, and a resolver comparing
/// the parsed name against what you asked for then never matched.
const RPM_ARCHES: &[&str] = &[
    "x86_64", "i686", "i386", "noarch", "aarch64", "armv7hl", "ppc64le", "s390x", "src",
];

/// Parses the output of `dnf search`.
///
/// Two shapes, because dnf5 (Fedora 41+) rewrote the output and dnf4 is still what RHEL and
/// older Fedora run: `name.arch : summary` and `name.arch<TAB>summary`. Both are the same
/// three facts with a different separator, so one pass reads either — and a header line, which
/// has neither separator, is dropped by the same rule rather than by a list of known headers.
pub fn parse_dnf_search(output: &str) -> Vec<Package> {
    sanitize(output)
        .lines()
        .filter_map(|line| {
            let (name_part, desc) = line.split_once('\t').or_else(|| line.split_once(" : "))?;
            let name = strip_rpm_arch(name_part.trim());
            if name.is_empty() {
                return None;
            }
            let mut p = Package::new(name, "dnf");
            p.properties
                .insert("description".to_string(), desc.trim().to_string());
            Some(p)
        })
        .collect()
}

fn strip_rpm_arch(name: &str) -> &str {
    match name.rsplit_once('.') {
        Some((base, arch)) if RPM_ARCHES.contains(&arch) => base,
        _ => name,
    }
}

/// Parses the table-based output of 'zypper search'.
/// Zypper output includes status indicators like 'i+' for installed.
///
/// **This is the parser whose failure mode the whole `Unrecognised` type exists for**, and its
/// comment below has said so since the `skip_while` bug: it is zypper's *installed* lister as
/// well as its search, so an output it cannot read is a mass-removal input. It can now say it
/// could not read one.
pub fn parse_zypper_search(output: &str) -> ParseResult {
    let clean = sanitize(output);
    let mut found: Vec<Package> = Vec::new();
    let mut candidates: Vec<&str> = Vec::new();
    // **The header is known by its position, not its words.** It is the table line directly
    // ABOVE the dashed rule zypper draws under itself — in ANY language. The old guard
    // dropped a row whose name cell read `Name`, so on a localized box (`S | Nom | Résumé …`)
    // the header sailed through as a phantom installed package named `Nom`, minted fresh on
    // every run.
    let lines: Vec<&str> = clean.lines().collect();
    let next_meaningful_is_rule = |i: usize| -> bool {
        lines[i + 1..]
            .iter()
            .find(|l| !l.trim().is_empty())
            .is_some_and(|l| l.trim_start().starts_with("---"))
    };
    for (i, raw) in lines.iter().enumerate() {
        let trimmed = raw.trim_start();
        if trimmed.starts_with("---") || trimmed.trim().is_empty() {
            continue;
        }
        if crate::parsers::is_prose_line(raw) {
            continue;
        }
        // The header: whatever sits directly above the rule. The lexical guard below stays
        // for the rule-less shape, where position cannot answer.
        let is_header =
            next_meaningful_is_rule(i) || matches!(parts_name(raw), Some(n) if n == "Name");
        if is_header {
            continue;
        }
        candidates.push(raw);
        // Table format: S | Name | Summary | Type
        let Some(name) = parts_name(raw) else {
            continue;
        };
        let parts: Vec<&str> = raw.split('|').collect();
        if parts.len() < 3 || name.is_empty() {
            continue;
        }
        let status = parts[0].trim();
        let summary = parts[2].trim();

        let mut p = Package::new(name, "zypper");
        p.properties
            .insert("summary".to_string(), summary.to_string());
        p.properties
            .insert("status_raw".to_string(), status.to_string());

        // If status contains 'i', it's already installed
        if status.contains('i') {
            p.properties.insert("installed".to_string(), "true".into());
        }

        found.push(p);
    }
    or_unrecognised("zypper", found, &candidates)
}

/// The cell between the first and second `|` of a zypper table row, trimmed.
fn parts_name(line: &str) -> Option<&str> {
    let parts: Vec<&str> = line.split('|').collect();
    if parts.len() < 2 {
        return None;
    }
    Some(parts[1].trim())
}

/// `dnf check-update -q` (`Q44`).
///
/// ```text
/// Upgrades
/// audit-libs.x86_64                  4.2.1-1.fc44     updates
/// coreutils.x86_64                   9.10-5.fc44      updates
/// ```
///
/// **The arch is stripped, because the installed listing strips it** — a row reported as
/// `audit-libs.x86_64` matches nothing the caller holds, so it would silently report zero
/// updates on a machine that has them.
///
/// `dnf check-update` exits **100** when it finds updates. That is an answer, not a fault, and
/// it is why this is read through a reader that tolerates a non-zero exit that still spoke.
pub fn parse_dnf_outdated(output: &str) -> Vec<Package> {
    sanitize(output)
        .lines()
        .filter_map(|line| {
            let mut f = line.split_whitespace();
            let name_arch = f.next()?;
            let version = f.next()?;
            // A section banner ("Upgrades", "Obsoleting Packages") has no second field.
            let _repo = f.next()?;
            let name = strip_rpm_arch(name_arch);
            // dnf wraps a long name onto its own line; the continuation has no dot-arch and
            // reads as a package with the version of whatever followed it.
            if name.is_empty() || !name_arch.contains('.') {
                return None;
            }
            Some(Package::with_version(name, version, "dnf"))
        })
        .collect()
}

/// `zypper --non-interactive list-updates` (`Q44`).
///
/// ```text
/// S  | Repository | Name         | Current Version | Available Version | Arch
/// ---+------------+--------------+-----------------+-------------------+-------
/// v  | repo-oss   | curl         | 8.17.0-1.1      | 8.18.0-1.1        | x86_64
/// ```
///
/// Six pipe-separated columns, and the one wanted is *Available*, not *Current*. Both are
/// versions and they sit next to each other, which is exactly the shape that gets read off by
/// one column and reports every package as already up to date.
pub fn parse_zypper_outdated(output: &str) -> Vec<Package> {
    sanitize(output)
        .lines()
        .filter(|l| !l.trim_start().starts_with("---"))
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() < 5 {
                return None;
            }
            let name = parts[2].trim();
            let available = parts[4].trim();
            // The header names its own columns and would otherwise become a package.
            if name.is_empty() || name == "Name" || available.is_empty() {
                return None;
            }
            Some(Package::with_version(name, available, "zypper"))
        })
        .collect()
}

/// `dnf repoquery --requires --resolve --queryformat %{name} <pkg>` — one bare package name per
/// line, and nothing else.
///
/// **A different shape from a labelled report, and so a different function.** `generic.rs`'s
/// labelled parser reads `Depends:`/`Requires:` rows and would answer this output with an empty
/// list, because there is no label on any line. One parser made lenient enough for both is one
/// parser that reads a malformed answer in either shape as a valid answer in the other — the
/// same rule `MachineListing` states for its own reader.
pub fn parse_bare_dependency_names(output: &str) -> Vec<String> {
    crate::utils::text::sanitize(output)
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect()
}
