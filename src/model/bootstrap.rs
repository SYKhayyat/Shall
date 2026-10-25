//! `adapters/bootstrap.toml` — how to obtain a package manager this machine does not have (7c).
//!
//! A config that declares `brew:ripgrep` on a machine with no Homebrew is not wrong; it is
//! ahead of the machine. Today that is an error the user resolves by going and reading
//! Homebrew's install page. **P8: Shall does the thing, it does not hand you the thing to do**
//! — so if the repo says how to get the manager, Shall offers to get it.
//!
//! **Ask, then do.** The command is printed in full and confirmed before it runs. That is the
//! shape P8 licenses and the one it does not: never *inform and leave*, never *act unasked*.
//! A bootstrap is usually `curl | sh` from a vendor, which is exactly the thing nobody should
//! run because a config file said so and nobody looked.
//!
//! The file goes through II.12's approval ledger like every other `adapters/` file, so the
//! command cannot arrive with a pulled repo and run unreviewed.

use crate::core::adapter::AdapterRow;
use serde::Deserialize;

/// One way to obtain one manager.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct BootstrapDef {
    /// The backend this obtains — the same name a line is written with (`brew`).
    pub manager: String,
    /// Restrict to one OS (`std::env::consts::OS`). Absent means any.
    #[serde(default)]
    pub os: Option<String>,
    /// The argv to run. A list, never a string: a string would have to be split by someone,
    /// and the someone is always a shell nobody declared (II.12b).
    pub run: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct BootstrapFile {
    #[serde(default)]
    pub bootstrap: Vec<BootstrapDef>,
}

impl BootstrapDef {
    /// The command as a reader would type it, for the confirmation. Printed in full and never
    /// abbreviated: the whole point of asking is that the reader sees what will run.
    pub fn command_line(&self) -> String {
        self.run.join(" ")
    }
}

impl AdapterRow for BootstrapDef {
    const WHAT: &'static str = "bootstrap row";
    const NAMELESS: &'static str = "it names no manager";

    /// Keyed by the manager it obtains. A file may carry a specific row and a general one for
    /// the same manager, so this table does not deduplicate by name — [`for_manager`] picks
    /// between them.
    fn name(&self) -> &str {
        &self.manager
    }

    fn only_on(&self) -> Option<&str> {
        self.os.as_deref()
    }

    /// A row that names no command describes nothing; running "the empty command" is not a
    /// thing to guess at.
    fn why_unusable(&self) -> Option<&'static str> {
        if self.run.iter().all(|a| a.trim().is_empty()) {
            return Some("its `run` command is empty");
        }
        None
    }
}

/// The row that obtains `manager` on this machine, if the repo carries one.
///
/// First match wins, and rows naming this OS are considered before catch-alls — a file may
/// carry a specific answer and a general one, and the specific one is the one that was written
/// for this machine.
pub fn for_manager<'a>(
    rows: &'a [BootstrapDef],
    manager: &str,
    os: &str,
) -> Option<&'a BootstrapDef> {
    let usable = |r: &&BootstrapDef| r.unusable().is_none() && r.manager == manager;
    rows.iter()
        .find(|r| usable(r) && r.os.is_some() && r.applies_to(os))
        .or_else(|| rows.iter().find(|r| usable(r) && r.os.is_none()))
}
