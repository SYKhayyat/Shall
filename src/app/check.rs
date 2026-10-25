//! `shall check` — the one command that looks at the machine (U9, 7i).
//!
//! There used to be ten commands answering "what is going on": `status`, `doctor`,
//! `unmanaged`, `absent`, `conflicts`, `audit` and the old `check`, each with its own
//! spelling, its own flags and its own idea of what a summary is. **Ruled 2026-07-24: they
//! collapse into one**, with a section per question, and the old names are deleted rather than
//! aliased (P2 — an alias is the second way to do one thing, kept alive).
//!
//! **`heal` survives, because it acts.** That is the whole dividing line: this command reads
//! and reports, and everything that repairs — including what `doctor --fix` used to do — is
//! `heal`'s. A command that both diagnoses and changes things is one you cannot run to find
//! out whether you want to change things.
//!
//! The default output is one line per section: a verdict, and the command that acts on it. A
//! section named on the command line prints its detail instead.

use std::collections::BTreeMap;
use std::fmt;

/// One question `check` can answer. Each is a section of the summary and a valid argument.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    /// Does everything the active profiles reach parse and resolve?
    Config,
    /// What would `sync` change?
    Drift,
    /// What would `adopt` take?
    Unmanaged,
    /// Which `absent:` declarations are in force?
    Absent,
    /// Is the same package declared two different ways?
    Conflicts,
    /// Can each backend actually run?
    Health,
    /// Does anything managed have a known vulnerability?
    Security,
    /// Is any code the repo can run (an event hook) unapproved, and so silently dead? (II.12)
    Approvals,
    /// Is any `adapters/*.toml` file written but not in use — malformed, unapproved, or holding
    /// rows of the wrong kind?
    ///
    /// **The other half of `Approvals`' reason.** A reader that cannot use its adapter file
    /// warns and carries on, which is the right call mid-`sync` on a working machine and the
    /// wrong place to learn about it: the warning scrolls past, the file stays inert, and the
    /// tool it was teaching Shall about goes unmanaged. Loud where loud is free.
    Adapters,
}

impl Section {
    pub const ALL: [Section; 9] = [
        Section::Config,
        Section::Drift,
        Section::Unmanaged,
        Section::Absent,
        Section::Conflicts,
        Section::Health,
        Section::Security,
        Section::Approvals,
        Section::Adapters,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Section::Config => "config",
            Section::Drift => "drift",
            Section::Unmanaged => "unmanaged",
            Section::Absent => "absent",
            Section::Conflicts => "conflicts",
            Section::Health => "health",
            Section::Security => "security",
            Section::Approvals => "approvals",
            Section::Adapters => "adapters",
        }
    }

    /// The one place the vocabulary is read from, so the parser and the error cannot disagree
    /// about what is legal.
    pub fn parse(name: &str) -> Option<Section> {
        let lower = name.trim().to_lowercase();
        Section::ALL.into_iter().find(|s| s.as_str() == lower)
    }

    pub fn vocabulary() -> String {
        Section::ALL
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl fmt::Display for Section {
    /// `f.pad`, not `f.write_str`: the summary aligns sections in a column, and `write_str`
    /// silently ignores the width in `{:<11}` — the padding is requested and does not happen.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(self.as_str())
    }
}

/// What one section found: a verdict line and, when there is something to do, the command that
/// does it. Naming the next command is the point — a report you have to translate into an
/// action yourself has done the easy half (P8).
#[derive(Debug, Clone)]
pub struct Finding {
    pub section: Section,
    /// True when nothing needs attention. Decides the summary marker and the exit code.
    pub ok: bool,
    /// The one-line verdict, e.g. "3 to install, 1 to remove".
    pub summary: String,
    /// What to run about it. `None` when there is nothing to do.
    pub next: Option<String>,
    /// The numbers `summary` states, in a form nothing has to read English to get.
    ///
    /// Every number a section reports goes here as well as into the sentence. Without it the
    /// only machine-readable thing in `check --json` was the `ok` flag, so `shall fleet` — whose
    /// entire job is "how far is each machine from its manifests" — could learn *that* a host
    /// had drifted and never *how much*. A consumer reaching for a count had to regex
    /// `"3 to install, 1 to remove"`, which makes the wording of a sentence an API.
    pub counts: BTreeMap<&'static str, usize>,
}

impl Finding {
    pub fn ok(section: Section, summary: impl Into<String>) -> Finding {
        Finding {
            section,
            ok: true,
            summary: summary.into(),
            next: None,
            counts: BTreeMap::new(),
        }
    }

    pub fn attention(
        section: Section,
        summary: impl Into<String>,
        next: impl Into<String>,
    ) -> Finding {
        Finding {
            section,
            ok: false,
            summary: summary.into(),
            next: Some(next.into()),
            counts: BTreeMap::new(),
        }
    }

    /// Record the numbers this section's sentence states.
    pub fn counting(mut self, counts: impl IntoIterator<Item = (&'static str, usize)>) -> Finding {
        self.counts.extend(counts);
        self
    }

    /// The summary line: section, verdict, and the command to run about it.
    pub fn line(&self) -> String {
        let mark = if self.ok { "ok  " } else { "->  " };
        let head = format!("{}{:<11} {}", mark, self.section, self.summary);
        match &self.next {
            Some(next) => format!("{}\n{:19}run `{}`", head, "", next),
            None => head,
        }
    }
}
