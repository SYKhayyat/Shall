//! `backup_adapters.toml` — the tools that can answer "when did a copy of this path last reach
//! TARGET?" (the `durability` section of `shall check`, #74).
//!
//! The answer the section needs is *not* "can this tool back things up" — it is "does a copy of
//! this exact path exist, and how old is the newest one". A tool whose listing cannot say which
//! paths a copy covers is refused by the probe rather than guessed at: an invariant satisfied on
//! the strength of "the repository has snapshots, presumably" is an invariant nobody checked.
//!
//! The checklist in `/backup_adapters.toml` documents the one shape a row may take: an argv that
//! prints a JSON array, each element carrying the moment the copy was made in a named field.

use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::core::adapter::{self, AdapterRow, Detected};

/// One row of `[[backup]]` in `backup_adapters.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct BackupAdapter {
    pub name: String,
    #[serde(default)]
    pub os: Option<String>,
    /// The command whose presence on `PATH` means this row describes this machine.
    pub detect: String,
    /// The argv that asks which copies of a path this target holds. `{path}` and `{target}` are
    /// filled by [`BackupAdapter::query`].
    pub list: Vec<String>,
    /// The field, on each element of the JSON array the listing prints, that carries when the
    /// copy was made (RFC 3339).
    pub time_field: String,
}

/// The file, as `toml` reads it — the table is `[[backup]]` (singular rows, plural file), the
/// same deliberate mismatch the other tables use so a typoed `[[backups]]` is visible noise
/// instead of a silently empty answer.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct BackupAdapterFile {
    #[serde(default)]
    pub backup: Vec<BackupAdapter>,
}

const BUILTIN: &str = include_str!("backup_adapters.toml");

impl AdapterRow for BackupAdapter {
    const WHAT: &'static str = "backup adapter";

    fn name(&self) -> &str {
        &self.name
    }

    fn only_on(&self) -> Option<&str> {
        self.os.as_deref()
    }

    fn why_unusable(&self) -> Option<&'static str> {
        if self.detect.trim().is_empty() {
            return Some("it has no `detect` command");
        }
        if self.list.is_empty() {
            return Some("it cannot list its copies, so nothing could be checked");
        }
        if self.time_field.trim().is_empty() {
            return Some("it does not say which field of its listing carries the time");
        }
        None
    }
}

impl Detected for BackupAdapter {
    fn detect_command(&self) -> &str {
        &self.detect
    }
}

impl BackupAdapter {
    /// The listing argv with `{path}` and `{target}` filled in.
    pub fn query(&self, path: &str, target: &str) -> Vec<String> {
        adapter::fill(&self.list, &[("{path}", path), ("{target}", target)])
    }

    /// The newest copy the listing names, or an error saying what was unreadable.
    ///
    /// `Ok(None)` is a listing that parsed and held no snapshot of this path — the shape of a
    /// path that was never backed up. `Err` is a listing that could not be read as restic's,
    /// which the probe reports by name: either answer is a fact, and the two are never merged
    /// into "something went wrong" (decision 3 of #74).
    pub fn newest(&self, stdout: &str) -> Result<Option<DateTime<Utc>>, String> {
        let rows: Vec<serde_json::Value> =
            serde_json::from_str(stdout).map_err(|e| format!("listing is not JSON: {e}"))?;
        let mut newest = None;
        for row in &rows {
            let Some(shown) = row
                .get(&self.time_field)
                .and_then(serde_json::Value::as_str)
            else {
                continue;
            };
            let parsed = DateTime::parse_from_rfc3339(shown)
                .map_err(|e| format!("`{}` is not a time: {e}", shown))?;
            let at = parsed.with_timezone(&Utc);
            if newest.is_none_or(|cur: DateTime<Utc>| at > cur) {
                newest = Some(at);
            }
        }
        Ok(newest)
    }
}

/// Every row in force: the shipped ones, then the user's (`adapters/backup.toml`), through the
/// same route every adapter table's rows take — shipped first, so a user row can never shadow
/// one Shall ships (K17/U1).
pub fn adapters(user_rows: Vec<BackupAdapter>) -> Vec<BackupAdapter> {
    let shipped: BackupAdapterFile =
        toml::from_str(BUILTIN).expect("the shipped backup_adapters.toml must parse");
    adapter::merge(shipped.backup.into_iter().chain(user_rows))
}

/// The row an invariant names, or `None` with a sentence about the ones that exist (#74's
/// "failed, named": a `backed-by` that names no row is a report, never a silent skip).
pub fn by_name<'a>(rows: &'a [BackupAdapter], name: &str) -> Option<&'a BackupAdapter> {
    rows.iter().find(|r| r.name.eq_ignore_ascii_case(name))
}
