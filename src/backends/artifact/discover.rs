//! Finding the executable inside an extracted archive.
//!
//! Operates on a listing rather than on a directory, so the rules are testable without a
//! filesystem and the walk stays in the backend where it belongs.

use std::fmt;
use std::path::{Path, PathBuf};

/// One file found inside the extracted archive, relative to its root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub path: PathBuf,
    /// Carries the executable bit on unix, or a `.exe`-shaped name on Windows.
    pub executable: bool,
}

impl Entry {
    pub fn new(path: impl Into<PathBuf>, executable: bool) -> Self {
        Entry {
            path: path.into(),
            executable,
        }
    }

    fn file_name(&self) -> String {
        self.path
            .file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscoveryError {
    /// Nothing in the archive looks like the program.
    NotFound { package: String, found: Vec<String> },
    /// Several do, and picking one would be a guess about which program you meant.
    Ambiguous {
        package: String,
        matches: Vec<String>,
    },
    /// `@bin=` named a path the archive does not contain.
    BinNotFound {
        requested: String,
        found: Vec<String>,
    },
}

impl fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiscoveryError::NotFound { package, found } => {
                write!(
                    f,
                    "{} — no executable found in the archive.{}\n  name it with @bin=PATH.",
                    package,
                    list(found)
                )
            }
            DiscoveryError::Ambiguous { package, matches } => write!(
                f,
                "{} — several executables in the archive, and they are different programs.{}\n  \
                 pick one with @bin=PATH.",
                package,
                list(matches)
            ),
            DiscoveryError::BinNotFound { requested, found } => write!(
                f,
                "@bin={} is not in the archive.{}",
                requested,
                list(found)
            ),
        }
    }
}

impl std::error::Error for DiscoveryError {}

fn list(items: &[String]) -> String {
    if items.is_empty() {
        return "\n  the archive is empty.".to_string();
    }
    let mut out = String::from("\n  it contains:");
    for item in items.iter().take(20) {
        out.push_str("\n    ");
        out.push_str(item);
    }
    if items.len() > 20 {
        out.push_str(&format!("\n    … and {} more", items.len() - 20));
    }
    out
}

/// `explicit` is `@bin=`. When it is given the guess is off entirely — that is the whole point
/// of the option, and a fallback would put the guess back exactly where the user turned it off.
pub fn find_executable(
    entries: &[Entry],
    package: &str,
    explicit: Option<&str>,
) -> Result<PathBuf, DiscoveryError> {
    if let Some(requested) = explicit {
        return find_named(entries, requested);
    }

    let stem = package.rsplit('/').next().unwrap_or(package).to_lowercase();

    for rule in [by_exact_name, by_name_prefix, by_lone_executable] {
        let matched = rule(entries, &stem);
        match matched.len() {
            0 => continue,
            1 => return Ok(matched[0].path.clone()),
            _ => {
                return Err(DiscoveryError::Ambiguous {
                    package: package.to_string(),
                    matches: display_all(&matched),
                })
            }
        }
    }

    Err(DiscoveryError::NotFound {
        package: package.to_string(),
        found: display_all(&entries.iter().collect::<Vec<_>>()),
    })
}

/// Matches on the full relative path first, then on the bare filename, so both
/// `@bin=build/fd` and `@bin=fd` work without the user knowing the archive's shape.
fn find_named(entries: &[Entry], requested: &str) -> Result<PathBuf, DiscoveryError> {
    let wanted = requested.replace('\\', "/").to_lowercase();
    let by_path = entries
        .iter()
        .find(|e| normalise(&e.path) == wanted)
        .or_else(|| entries.iter().find(|e| e.file_name() == wanted));

    by_path
        .map(|e| e.path.clone())
        .ok_or_else(|| DiscoveryError::BinNotFound {
            requested: requested.to_string(),
            found: display_all(&entries.iter().collect::<Vec<_>>()),
        })
}

pub fn normalise(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/").to_lowercase()
}

fn display_all(entries: &[&Entry]) -> Vec<String> {
    entries.iter().map(|e| normalise(&e.path)).collect()
}

fn by_exact_name<'a>(entries: &'a [Entry], stem: &str) -> Vec<&'a Entry> {
    let with_exe = format!("{}.exe", stem);
    entries
        .iter()
        .filter(|e| e.file_name() == stem || e.file_name() == with_exe)
        .collect()
}

/// `fd-v10` in an archive named for its version. A dot rules it out: `fd.1` is a man page and
/// `fd.bash` a completion script, neither of which is the program.
fn by_name_prefix<'a>(entries: &'a [Entry], stem: &str) -> Vec<&'a Entry> {
    entries
        .iter()
        .filter(|e| {
            let name = e.file_name();
            name.starts_with(stem) && !name.contains('.')
        })
        .collect()
}

fn by_lone_executable<'a>(entries: &'a [Entry], _stem: &str) -> Vec<&'a Entry> {
    entries.iter().filter(|e| e.executable).collect()
}
