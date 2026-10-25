//! Backend groups — a name for a chain of backends (U18).
//!
//! `apt,dnf,cargo:ripgrep` on every line is repetition. A group is a shorthand: define
//! `tools = apt, dnf, cargo` once, then write `tools:ripgrep` and it expands to that chain.
//!
//! **It is only a shortcut.** A group expands to exactly the comma-chain you would have typed,
//! so it inherits that chain's meaning and its safety with nothing added — `priority` still
//! exists, a bare name still resolves through it, and `tools:ripgrep` resolves the same way
//! `apt,dnf,cargo:ripgrep` already does. The expansion happens in the one grammar that parses a
//! prefix (V: "one parser for `backend:name`"), so there is no second place that splits on `:`.
//!
//! Pure: parsing the file and expanding a name. A group's *members* are validated as real
//! backends by the grammar, not here — this only records what was written.

use std::collections::BTreeMap;

/// The `groups` file: `name = backend, backend, …`, one per line.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Groups {
    by_name: BTreeMap<String, Vec<String>>,
}

impl Groups {
    /// Parse a `groups` file. A blank line or a `#` comment is skipped; anything else must be
    /// `name = a, b, c`. A malformed line is reported, not guessed at — a group nobody can read
    /// is a prefix that silently means nothing.
    ///
    /// **A member may be another group** — groups nest (owner, 2026-07-24). Nested groups are
    /// flattened here, once, into the ultimate backend list, so the grammar only ever splices
    /// terminal names. A group that reaches itself, directly or through others, is a **cycle**
    /// and an error — the same refusal a `use` loop gets, and for the same reason: it has no
    /// answer.
    pub fn parse(text: &str) -> Result<Groups, String> {
        // Raw definitions first: a member may name a group defined later in the file, so every
        // definition has to be read before any can be flattened.
        //
        // The line hygiene is `config::without_bom` + `grammar::strip_comment`, the same pair
        // the modules reader takes. This reader used to split on the FIRST `#` anywhere, so a
        // group naming `foo#bar` lost its tail — and a BOM'd file failed with a refusal about
        // the wrong thing entirely, the first line's name carrying an invisible character.
        let mut raw: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for (i, line_raw) in crate::config::without_bom(text).lines().enumerate() {
            let line = crate::config::grammar::strip_comment(line_raw).trim();
            if line.is_empty() {
                continue;
            }
            let Some((name, members)) = line.split_once('=') else {
                return Err(format!(
                    "groups:{}: `{}` is not `name = backend, backend`",
                    i + 1,
                    line_raw.trim()
                ));
            };
            let name = name.trim();
            if name.is_empty() {
                return Err(format!("groups:{}: a group with no name", i + 1));
            }
            let members: Vec<String> = members
                .split(',')
                .map(str::trim)
                .filter(|m| !m.is_empty())
                .map(str::to_string)
                .collect();
            if members.is_empty() {
                return Err(format!(
                    "groups:{}: group `{}` names no backends",
                    i + 1,
                    name
                ));
            }
            if raw.insert(name.to_string(), members).is_some() {
                return Err(format!(
                    "groups:{}: group `{}` is defined twice",
                    i + 1,
                    name
                ));
            }
        }

        // Flatten each group into terminal backends, following nested groups and refusing a
        // cycle.
        let mut by_name = BTreeMap::new();
        for name in raw.keys() {
            let mut path = Vec::new();
            let members = flatten(name, &raw, &mut path)?;
            by_name.insert(name.clone(), members);
        }
        Ok(Groups { by_name })
    }

    /// Load from disk. A missing file is no groups — the ordinary state, never an error.
    pub fn load(path: &std::path::Path) -> Result<Groups, String> {
        match std::fs::read_to_string(path) {
            Ok(text) => Groups::parse(&text),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Groups::default()),
            Err(e) => Err(format!("reading {}: {}", path.display(), e)),
        }
    }

    /// The backends a group names, in order — or `None` if the name is not a group.
    pub fn expand(&self, name: &str) -> Option<&[String]> {
        self.by_name.get(name).map(Vec::as_slice)
    }

    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.by_name.keys()
    }

    pub fn is_empty(&self) -> bool {
        self.by_name.is_empty()
    }
}

/// Flatten one group into its ultimate backend list: a member that names another group is
/// followed, a member that does not is a terminal backend. Duplicates are dropped keeping the
/// first (so two groups that overlap union cleanly rather than tripping the grammar's
/// "named twice" — that check is for a hand-written chain's typo, not a group union). A group
/// reached twice on one path is a cycle.
fn flatten(
    name: &str,
    raw: &BTreeMap<String, Vec<String>>,
    path: &mut Vec<String>,
) -> Result<Vec<String>, String> {
    if path.iter().any(|p| p == name) {
        path.push(name.to_string());
        return Err(format!(
            "group `{}` reaches itself: {}. A group cannot contain itself.",
            name,
            path.join(" → ")
        ));
    }
    path.push(name.to_string());
    let mut out: Vec<String> = Vec::new();
    for member in raw.get(name).into_iter().flatten() {
        if raw.contains_key(member) {
            for backend in flatten(member, raw, path)? {
                if !out.contains(&backend) {
                    out.push(backend);
                }
            }
        } else if !out.contains(member) {
            out.push(member.clone());
        }
    }
    path.pop();
    Ok(out)
}
