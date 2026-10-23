use crate::core::Package;
use crate::parsers::{or_unrecognised, ParseResult};
use crate::utils::text::sanitize;

/// A generic parser for backends that return a simple space-separated list.
/// Format: "package-name version" or just "package-name"
/// Used by backends like 'apk' or internal search utilities.
pub fn parse_simple_list(output: &str, backend: &str) -> ParseResult {
    let clean = sanitize(output);
    let candidates = crate::parsers::data_lines(&clean);
    let found = candidates
        .iter()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                Some(Package::with_version(parts[0], parts[1], backend))
            } else if parts.len() == 1 {
                Some(Package::new(parts[0], backend))
            } else {
                None
            }
        })
        .collect();
    or_unrecognised(backend, found, &candidates)
}

/// Parses a list where the version is attached to the name via a dash.
///
/// `package-name-1.2.3-r1` -> name `package-name`, version `1.2.3-r1`. Alpine's `apk info -v`
/// and `apk search -v` both answer this way; the search form adds ` - description` after the
/// token, so only the first whitespace-separated field is read.
///
/// **The version half must start with a digit.** Package names contain dashes — `xz-libs-dev`
/// split blind gives name `xz` and version `libs-dev`, and since this is apk's installed
/// lister, a package under the wrong name can never be matched by `info()`, so `remove`
/// silently does nothing. `xbps` and `pkgsrc` already parse this shape with the digit check;
/// this is the same rule, not a new one.
pub fn parse_dash_version_list(output: &str, backend: &str) -> ParseResult {
    let clean = sanitize(output);
    let candidates = crate::parsers::data_lines(&clean);
    let found = candidates
        .iter()
        .filter_map(|line| {
            let token = line.split_whitespace().next()?;
            Some(match split_trailing_version(token) {
                Some((name, version)) => Package::with_version(name, &version, backend),
                None => Package::new(token, backend),
            })
        })
        .collect();
    or_unrecognised(backend, found, &candidates)
}

/// `name-1.2.3-r1` -> `("name", "1.2.3-r1")`, and `None` when no trailing field looks like a
/// version. The scan is right-to-left so the *last* dash-joined version wins.
pub(crate) fn split_trailing_version(token: &str) -> Option<(&str, String)> {
    let starts_with_digit = |s: &str| s.chars().next().is_some_and(|c| c.is_ascii_digit());

    // `name-1.2.3-r1`: the revision is not itself a version, so try two fields before one.
    if let Some((head, rev)) = token.rsplit_once('-') {
        if let Some((name, ver)) = head.rsplit_once('-') {
            if starts_with_digit(ver) && !name.is_empty() {
                return Some((name, format!("{}-{}", ver, rev)));
            }
        }
        if starts_with_digit(rev) && !head.is_empty() {
            return Some((head, rev.to_string()));
        }
    }
    None
}

/// `apk version -l '<'` (`Q44`): `name-installed < available`, under a two-column header.
///
/// ```text
/// Installed:                                Available:
/// apk-tools-3.0.6-r0                      < 3.0.7-r0
/// ```
///
/// The left side is a `name-version` token, split by the same rule the installed listing uses
/// — a different rule here would produce names that match nothing the caller holds.
pub fn parse_apk_outdated(output: &str, backend: &str) -> Vec<Package> {
    sanitize(output)
        .lines()
        .filter_map(|line| {
            let (installed, available) = line.split_once('<')?;
            let token = installed.split_whitespace().next()?;
            let available = available.trim();
            if available.is_empty() {
                return None;
            }
            let (name, _) = split_trailing_version(token)?;
            Some(Package::with_version(name, available, backend))
        })
        .collect()
}

/// `brew outdated --json=v2` (`Q44`).
///
/// ```json
/// {"formulae":[{"name":"jq","installed_versions":["1.7"],"current_version":"1.7.1"}],"casks":[]}
/// ```
///
/// **The JSON is found, not assumed.** brew prints `==> Auto-updating Homebrew...` and its
/// environment hints ahead of the payload, so parsing the whole output fails and would report
/// nothing outdated on precisely the machines that had not run brew recently. Casks are read
/// too: a cask is installed by the same declaration and its updates are updates.
pub fn parse_brew_outdated(output: &str) -> Vec<Package> {
    let text = sanitize(output);
    let Some(start) = text.find('{') else {
        return Vec::new();
    };
    let Ok(doc) = serde_json::from_str::<serde_json::Value>(&text[start..]) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for key in ["formulae", "casks"] {
        let Some(items) = doc.get(key).and_then(|f| f.as_array()) else {
            continue;
        };
        for item in items {
            let Some(name) = item.get("name").and_then(|n| n.as_str()) else {
                continue;
            };
            let name = name.trim();
            if name.is_empty() {
                continue;
            }
            let current = item
                .get("current_version")
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|v| !v.is_empty());
            out.push(match current {
                Some(v) => Package::with_version(name, v, "brew"),
                None => Package::new(name, "brew"),
            });
        }
    }
    out
}
