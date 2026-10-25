//! `@asset=` — the escape hatch that narrows the candidate list by filename.
//!
//! A pattern narrows; it never selects. When it still matches several assets the ordinary
//! tie-break decides, so there is exactly one rule for "two assets, both legal".

use regex::Regex;
use std::fmt;

#[derive(Debug, Clone)]
pub enum AssetPattern {
    /// Every asset that survives the format filter is installed, rather than one being chosen.
    All,
    Glob {
        source: String,
        compiled: Regex,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BadPattern {
    pub given: String,
    pub reason: String,
}

impl fmt::Display for BadPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid @asset={}: {}", self.given, self.reason)
    }
}

impl std::error::Error for BadPattern {}

impl AssetPattern {
    /// Whether this pattern is the asset's literal name rather than a glob that happens to
    /// match it. Naming a file exactly is a claim the user is making about it — which is what
    /// lets `@asset=` install an extension-less binary whose name says nothing about the
    /// platform (D2).
    pub fn names_exactly(&self, name: &str) -> bool {
        match self {
            AssetPattern::All => false,
            AssetPattern::Glob { source, .. } => source == name && !source.contains(['*', '?']),
        }
    }

    pub fn parse(value: &str) -> Result<Self, BadPattern> {
        let value = value.trim();
        if value.is_empty() {
            return Err(BadPattern {
                given: value.to_string(),
                reason: "an empty pattern matches nothing. Give a filename, a glob like \
                         *musl*, or `all`."
                    .to_string(),
            });
        }
        if value.eq_ignore_ascii_case("all") {
            return Ok(AssetPattern::All);
        }
        let compiled = Regex::new(&glob_to_regex(value)).map_err(|e| BadPattern {
            given: value.to_string(),
            reason: e.to_string(),
        })?;
        Ok(AssetPattern::Glob {
            source: value.to_string(),
            compiled,
        })
    }

    pub fn matches(&self, filename: &str) -> bool {
        match self {
            AssetPattern::All => true,
            AssetPattern::Glob { compiled, .. } => compiled.is_match(&filename.to_lowercase()),
        }
    }

    pub fn installs_every_match(&self) -> bool {
        matches!(self, AssetPattern::All)
    }
}

impl fmt::Display for AssetPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AssetPattern::All => f.write_str("all"),
            AssetPattern::Glob { source, .. } => f.write_str(source),
        }
    }
}

/// Only `*` and `?` are wildcards. Everything else is escaped, so a `.` in a filename is a dot
/// and not "any character" — release filenames are full of dots and a user typing one means it.
fn glob_to_regex(glob: &str) -> String {
    let mut out = String::with_capacity(glob.len() * 2 + 4);
    out.push('^');
    for ch in glob.to_lowercase().chars() {
        match ch {
            '*' => out.push_str(".*"),
            '?' => out.push('.'),
            other => out.push_str(&regex::escape(&other.to_string())),
        }
    }
    out.push('$');
    out
}
