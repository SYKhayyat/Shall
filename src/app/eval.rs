//! `shall eval` — the resolved desired state, as data (XIII.15, U17).
//!
//! Everything else in Shall answers a question about the machine. This answers the question
//! *what did my configuration actually resolve to* — every `when` decided, every bare name
//! given a backend, every variable substituted — without looking at the machine at all.
//!
//! **Versioned from the first release (U17, ruled 2026-07-24).** It is the one output that will
//! acquire consumers Shall cannot see, and P2 leaves no legacy to carry: the version is free
//! now and impossible to add later without breaking whatever grew up around it.
//!
//! **It takes no locks and changes nothing.** A command that reads has no business serialising
//! against one that writes, and a command people will put in a pipeline must be safe to run
//! while a sync is in flight.

use serde::Serialize;

/// The schema version of `shall eval`'s output.
///
/// Bump when a consumer would break: a field removed, renamed, or given a new meaning. Adding a
/// field is not a bump — a consumer that ignores unknown keys keeps working, and one that does
/// not was never going to survive any change.
pub const SCHEMA: u32 = 1;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Evaluation {
    /// The schema this document is written to. First field, so a consumer can branch on it
    /// before reading anything whose meaning depends on it.
    pub schema: u32,
    /// The variables every `when` was decided against (Part IX), with their types intact — a
    /// `Num` stays a number, so a consumer is not handed `"3"` and left to guess.
    pub vars: std::collections::BTreeMap<String, crate::model::vars::Value>,
    /// What must be installed, resolved: a backend for every name, in a stable order.
    pub present: Vec<ResolvedPackage>,
    /// What must not exist (`absent:`).
    pub absent: Vec<ResolvedPackage>,
    /// The non-package declarations — `service:nginx`, `firewall:22/tcp`.
    pub extras: Vec<ResolvedExtra>,
    /// Lines whose `@until` has passed (II.16). They are not in `present`, and a consumer that
    /// only read `present` would never learn that a declaration silently stopped counting.
    pub lapsed: Vec<Lapsed>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResolvedPackage {
    pub backend: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// The file and line that declared it. The whole value of this output is being able to ask
    /// "why is this here", and an answer with no origin cannot be checked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResolvedExtra {
    pub kind: String,
    pub name: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Lapsed {
    pub line: String,
    pub source: String,
}

/// A `file:line` origin as this document reports it: relative to the config repo, with forward
/// slashes.
///
/// Two evaluations are meant to be diffed — the same repo on two machines, or before and after
/// an edit. An absolute path makes every line differ because the repo is checked out somewhere
/// else, and a backslash makes every line differ because one machine is Windows. Neither
/// difference is one the configuration made, so neither belongs in the output.
pub fn repo_relative(source: &str, root: &std::path::Path) -> String {
    let (path, line) = match source.rsplit_once(':') {
        // A drive letter is not a line number: `C:/x` must not split into `C` and `/x`.
        Some((p, n)) if n.chars().all(|c| c.is_ascii_digit()) && !n.is_empty() => (p, Some(n)),
        _ => (source, None),
    };
    let rel = std::path::Path::new(path)
        .strip_prefix(root)
        .unwrap_or_else(|_| std::path::Path::new(path))
        .to_string_lossy()
        .replace('\\', "/");
    match line {
        Some(n) => format!("{}:{}", rel, n),
        None => rel,
    }
}

impl Evaluation {
    /// Build the document from a resolved model.
    ///
    /// Sorted throughout: a consumer diffing two evaluations wants the difference to be the
    /// configuration's, not the order a `HashMap` happened to iterate in.
    pub fn of(state: &crate::model::DesiredState, config_root: &std::path::Path) -> Evaluation {
        let package = |p: &crate::core::PackageSpec| ResolvedPackage {
            backend: p.backend.clone(),
            name: p.name.clone(),
            version: p.options.one("version").map(str::to_string),
            source: p
                .options
                .one("__source")
                .map(|s| repo_relative(s, config_root)),
        };

        let mut present: Vec<ResolvedPackage> = state.present().map(package).collect();
        let mut absent: Vec<ResolvedPackage> = state.absent().map(package).collect();
        present.sort();
        absent.sort();

        // Only statements with a keyword: a package line's prefix is a backend, and it is
        // already reported above under `present`/`absent`.
        let mut extras: Vec<ResolvedExtra> = state
            .extras
            .iter()
            .filter_map(|(stmt, origin)| {
                Some(ResolvedExtra {
                    kind: stmt.kind()?.to_string(),
                    name: stmt.subject()?,
                    source: repo_relative(&origin.to_string(), config_root),
                })
            })
            .collect();
        extras.sort();

        let mut lapsed: Vec<Lapsed> = state
            .lapsed
            .iter()
            .map(|(line, origin)| Lapsed {
                line: line.clone(),
                source: repo_relative(&origin.to_string(), config_root),
            })
            .collect();
        lapsed.sort();

        Evaluation {
            schema: SCHEMA,
            vars: state.vars.clone(),
            present,
            absent,
            extras,
            lapsed,
        }
    }

    /// The document as the bytes `shall eval` writes.
    ///
    /// Pretty-printed with a trailing newline: this output is read by people at least as often
    /// as by programs, and a JSON document without a final newline is the kind of thing that
    /// makes a shell prompt land in the middle of it.
    pub fn render(&self) -> crate::core::Result<String> {
        let body = serde_json::to_string_pretty(self)
            .map_err(|e| crate::core::Error::Other(format!("serializing the evaluation: {}", e)))?;
        Ok(format!("{}\n", body))
    }
}
