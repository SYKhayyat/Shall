//! `invariant:NAME @path=…,backed-by=ADAPTER:TARGET,max-age=…` — a promise about this machine
//! that no sync can converge (#74).
//!
//! **Not a resource.** Nothing is placed, so there is nothing to undo, nothing in the extras
//! ledger, and no phase. What it buys is a question `check` asks: does a copy of this path exist
//! somewhere else, and is the newest one younger than the promise allows? The answer is read off
//! the backup tool's own listing — never off a file Shall wrote, because a receipt Shall wrote is
//! the one thing a broken backup cannot fail to produce.

use crate::config::grammar::{GrammarError, Options, Origin, Result};
use chrono::Duration;

/// One parsed `invariant:` line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invariant {
    /// What the promise is called, for a report that has to name it.
    pub name: String,
    /// The path whose existence elsewhere is promised, as written after `$home` and friends
    /// were substituted.
    pub path: String,
    /// Which backup tool's listing answers the question (`restic`, and any adapter row a user
    /// adds).
    pub adapter: String,
    /// Where that tool keeps the copy — the repository for `restic`, a `user@host:/dir` for the
    /// adapters that take one.
    pub target: String,
    /// How old the newest copy may be before the promise is broken.
    pub max_age: Duration,
}

impl Invariant {
    /// Parse and check one line.
    ///
    /// Called from grammar validation while the file is being read and again when the state is
    /// resolved, so a bad line is refused at the file rather than at `check` time — with one
    /// implementation, so the two can never disagree about what the line means.
    pub fn parse(origin: &Origin, name: &str, options: &Options) -> Result<Self> {
        if name.trim().is_empty() {
            return Err(GrammarError::new(
                origin.clone(),
                "`invariant:` needs a name, as in `invariant:seforim @path=…`".to_string(),
            )
            .with_hint(
                "the name is what a report calls the promise, so it has to be there to be \
                 reported.",
            ));
        }
        let field = |key: &str| -> Result<String> {
            match options.one(key) {
                Some(value) if !value.trim().is_empty() => Ok(value.trim().to_string()),
                Some(_) => Err(GrammarError::new(
                    origin.clone(),
                    format!("`invariant:{}` has an empty `@{}`", name, key),
                )
                .with_hint(format!(
                    "`@{}` says where the promise is checked; an empty one checks nothing.",
                    key
                ))),
                None => Err(GrammarError::new(
                    origin.clone(),
                    format!("`invariant:{}` is missing `@{}`", name, key),
                )
                .with_hint(
                    "an invariant says three things: `@path=`, `@backed-by=ADAPTER:TARGET` and \
                     `@max-age=`, as in `invariant:seforim @path=$home/Documents/seforim,\
                     backed-by=restic:b2:seforim,max-age=7d`.",
                )),
            }
        };
        let path = field("path")?;
        let backed_by = field("backed-by")?;
        let max_age = parse_max_age(origin, name, &field("max-age")?)?;
        let (adapter, target) = match backed_by.split_once(':') {
            Some((adapter, target)) if !adapter.trim().is_empty() && !target.trim().is_empty() => {
                (adapter.trim().to_string(), target.trim().to_string())
            }
            _ => {
                return Err(GrammarError::new(
                    origin.clone(),
                    format!(
                        "`@backed-by={}` does not say which tool and where",
                        backed_by
                    ),
                )
                .with_hint(
                    "write `@backed-by=ADAPTER:TARGET`, as in `@backed-by=restic:b2:seforim`. \
                     The part before the first `:` names the backup tool, so it decides whose \
                     listing is believed.",
                ))
            }
        };
        Ok(Self {
            name: name.to_string(),
            path,
            adapter,
            target,
            max_age,
        })
    }
}

/// `7d`, `36h`, `90m` — how long a copy may sit before the promise is broken.
///
/// Deliberately three units and no word for "forever": an invariant with no deadline is a line
/// that is satisfied by a copy of anything from any year, which is the shape of a promise nobody
/// checked.
fn parse_max_age(origin: &Origin, name: &str, written: &str) -> Result<Duration> {
    let (digits, unit) = written.split_at(
        written
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(written.len()),
    );
    let count: i64 = digits.parse().map_err(|_| {
        GrammarError::new(
            origin.clone(),
            format!("`@max-age={}` is not a length of time", written),
        )
        .with_hint("write a number and a unit: `7d`, `36h` or `90m`.")
    })?;
    let per_unit = match unit {
        "d" => 24 * 60 * 60,
        "h" => 60 * 60,
        "m" => 60,
        _ => {
            return Err(GrammarError::new(
                origin.clone(),
                format!("`@max-age={}` is not a length of time", written),
            )
            .with_hint(format!(
                "`{}` is not a unit; `invariant:{}` takes `d`, `h` or `m`, as in `@max-age=7d`.",
                unit, name
            )))
        }
    };
    if count <= 0 {
        return Err(GrammarError::new(
            origin.clone(),
            format!("`@max-age={}` allows no time at all", written),
        )
        .with_hint(
            "a copy younger than zero seconds does not exist, so the promise could only be kept \
             by a lie.",
        ));
    }
    Ok(Duration::seconds(count.saturating_mul(per_unit)))
}
