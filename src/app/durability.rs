//! Does a copy of the promised path exist elsewhere, and how old is it? (#74)
//!
//! The question is asked of the **backup tool's own listing**, never of anything Shall wrote.
//! A receipt Shall produced would say the copy was made — and it would say so just as loudly on
//! the machine whose repository password expired three weeks ago, which is the one case the
//! promise exists for.
//!
//! Every answer is one of four, and they are kept apart on purpose: a copy younger than the
//! promise, a copy older than it, no copy at all, and a question nothing on this machine could
//! answer. Folding the last into either of the others is how a check reports success over a
//! measurement it never took.

use chrono::Utc;

use crate::app::check::{Finding, Section};
use crate::backends::backup::BackupAdapter;
use crate::config::grammar::Origin;
use crate::core::adapter;
use crate::core::CommandExecutor;
use crate::model::invariant::Invariant;

/// What one invariant's question came back as.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// A copy exists and is younger than `@max-age`.
    Fresh { age_secs: i64 },
    /// The newest copy is older than `@max-age`.
    Stale { age_secs: i64 },
    /// The tool answered, and no copy of this path is in it.
    Never,
    /// Nothing here could ask, or what answered could not be read. Carries the sentence a
    /// reader needs — never a bare boolean, because "unanswerable" without its reason is a
    /// fact you cannot act on.
    Unanswerable(String),
}

impl Verdict {
    /// Whether this answer keeps the promise.
    pub fn holds(&self) -> bool {
        matches!(self, Verdict::Fresh { .. })
    }
}

/// One invariant, measured.
#[derive(Debug, Clone)]
pub struct Answer {
    pub name: String,
    pub path: String,
    pub adapter: String,
    pub target: String,
    pub max_age_secs: i64,
    pub verdict: Verdict,
}

impl Answer {
    /// The one line a report prints for this promise.
    pub fn describe(&self) -> String {
        let promise = format!("{} — {}", self.path, self.promise());
        match &self.verdict {
            Verdict::Fresh { age_secs } => format!(
                "{}: {promise}, newest copy {}",
                self.name,
                human_age(*age_secs)
            ),
            Verdict::Stale { age_secs } => format!(
                "{}: {promise}, but the newest copy is {}",
                self.name,
                human_age(*age_secs)
            ),
            Verdict::Never => format!(
                "{}: {promise}, and `{}` holds no copy of it",
                self.name, self.target
            ),
            Verdict::Unanswerable(why) => format!("{}: {promise}, but {why}", self.name),
        }
    }

    fn promise(&self) -> String {
        format!(
            "a copy in `{}` (via `{}`) at most {} old",
            self.target,
            self.adapter,
            human_span(self.max_age_secs)
        )
    }
}

/// `7d`, `36h`, `90m` — how long a promise allows, in the unit the line was written in.
fn human_span(secs: i64) -> String {
    if secs % 86_400 == 0 {
        format!("{}d", secs / 86_400)
    } else if secs % 3_600 == 0 {
        format!("{}h", secs / 3_600)
    } else {
        format!("{}m", secs / 60)
    }
}

/// `2d`, `5h`, `40m` — an age in the unit a person would say it in.
fn human_age(secs: i64) -> String {
    let secs = secs.max(0);
    if secs >= 86_400 {
        format!("{}d old", secs / 86_400)
    } else if secs >= 3_600 {
        format!("{}h old", secs / 3_600)
    } else {
        format!("{}m old", secs / 60)
    }
}

/// Measure every invariant, in declaration order.
///
/// Takes the executor and the rows rather than an `App`, so the whole measurement is driven by a
/// mock in a test and none of it needs a repository on the far side of the network.
pub async fn measure(
    executor: &CommandExecutor,
    rows: &[BackupAdapter],
    invariants: &[(Invariant, Origin)],
) -> Vec<Answer> {
    let now = Utc::now();
    let mut out = Vec::new();
    for (inv, _origin) in invariants {
        let verdict = ask(executor, rows, inv, now).await;
        out.push(Answer {
            name: inv.name.clone(),
            path: inv.path.clone(),
            adapter: inv.adapter.clone(),
            target: inv.target.clone(),
            max_age_secs: inv.max_age.num_seconds(),
            verdict,
        });
    }
    out
}

async fn ask(
    executor: &CommandExecutor,
    rows: &[BackupAdapter],
    inv: &Invariant,
    now: chrono::DateTime<Utc>,
) -> Verdict {
    let Some(row) = crate::backends::backup::by_name(rows, &inv.adapter) else {
        let known: Vec<&str> = rows.iter().map(|r| r.name.as_str()).collect();
        return Verdict::Unanswerable(format!(
            "no backup adapter is called `{}` — the ones here are {} (add a `[[backup]]` row to \
             `adapters/backup.toml`)",
            inv.adapter,
            if known.is_empty() {
                "none".to_string()
            } else {
                known.join(", ")
            }
        ));
    };
    if !executor.command_exists(&row.detect).await {
        return Verdict::Unanswerable(format!(
            "`{}` is not installed, so nothing here can say whether a copy exists",
            row.detect
        ));
    }
    // `~` is a path a person writes and a tool cannot read: `restic --path ~/Documents` looks for
    // a directory literally called `~`. Expanded here through the same account helper `link:`
    // uses, so the two agree about what `~` means.
    let path = match crate::core::account::Account::current() {
        Ok(account) => match account.expand_home(&inv.path) {
            Ok(path) => path.to_string_lossy().to_string(),
            Err(e) => {
                return Verdict::Unanswerable(format!(
                    "`{}` could not be turned into a path: {e}",
                    inv.path
                ))
            }
        },
        Err(e) => {
            return Verdict::Unanswerable(format!(
                "no home directory to resolve `{}` against: {e}",
                inv.path
            ))
        }
    };
    let Some((program, args)) = adapter::program_and_args(row.query(&path, &inv.target)) else {
        return Verdict::Unanswerable(format!(
            "the `{}` row's `list` has a placeholder nothing fills",
            row.name
        ));
    };
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let stdout = match executor.probe_output(&program, &refs).await {
        Ok(stdout) => stdout,
        // The command ran and refused. Its own words are the answer: a wrong password, an
        // unreachable repository and a missing config all land here, and they need different
        // fixes.
        Err(e) => {
            return Verdict::Unanswerable(format!(
                "`{} -r {}` could not answer: {}",
                program,
                inv.target,
                first_line(&e.to_string())
            ))
        }
    };
    match row.newest(&stdout) {
        Err(why) => Verdict::Unanswerable(format!(
            "the `{}` listing could not be read: {why}",
            row.name
        )),
        Ok(None) => Verdict::Never,
        Ok(Some(at)) => {
            let age = now.signed_duration_since(at);
            if age > inv.max_age {
                Verdict::Stale {
                    age_secs: age.num_seconds(),
                }
            } else {
                Verdict::Fresh {
                    age_secs: age.num_seconds(),
                }
            }
        }
    }
}

/// The first line of an error, because a command's stderr is usually three of them and the
/// sentence this goes into is one.
fn first_line(message: &str) -> String {
    message.lines().next().unwrap_or(message).trim().to_string()
}

/// The `durability` row of `shall check`, built by appending what is true.
///
/// A machine that has declared no invariant is `ok`, not silent: "nothing promised" is the
/// ordinary case and a check that skipped the line would leave a reader unsure whether the
/// section ran at all.
pub fn finding(answers: &[Answer]) -> Finding {
    if answers.is_empty() {
        return Finding::ok(Section::Durability, "no invariant declared")
            .counting([("invariants", 0)]);
    }
    let count =
        |wanted: fn(&Verdict) -> bool| answers.iter().filter(|a| wanted(&a.verdict)).count();
    let fresh = count(|v| matches!(v, Verdict::Fresh { .. }));
    let stale = count(|v| matches!(v, Verdict::Stale { .. }));
    let never = count(|v| matches!(v, Verdict::Never));
    let unanswerable = count(|v| matches!(v, Verdict::Unanswerable(_)));
    let counts = [
        ("invariants", answers.len()),
        ("fresh", fresh),
        ("stale", stale),
        ("never", never),
        ("unanswerable", unanswerable),
    ];
    if fresh == answers.len() {
        return Finding::ok(
            Section::Durability,
            format!(
                "{} invariant(s) hold — a copy of each exists and is younger than promised",
                answers.len()
            ),
        )
        .counting(counts);
    }
    let mut parts = Vec::new();
    if stale > 0 {
        parts.push(format!("{stale} older than promised"));
    }
    if never > 0 {
        parts.push(format!("{never} with no copy in the target"));
    }
    if unanswerable > 0 {
        parts.push(format!("{unanswerable} nothing could answer"));
    }
    Finding::attention(
        Section::Durability,
        format!(
            "{} of {} invariant(s) broken — {}",
            answers.len() - fresh,
            answers.len(),
            parts.join(", ")
        ),
        "shall check durability",
    )
    .counting(counts)
}
