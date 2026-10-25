//! What an `exec:` or a `generate:` line may say.
//!
//! The two verb statements, kept together and kept out of the parser. Every other keyword in
//! this grammar is a noun whose validation is a table of option keys; these two are the pair
//! that carry real rules — which verb a step belongs to, how many times its content may run,
//! how many packages must have moved first — and those rules are what grew `statement.rs` past
//! the size at which one file is one subject. `model::step::refusal` left for the same reason
//! and is called from here.

use super::error::{GrammarError, Origin, Result};
use super::options::Options;
use super::statement::{validate_extra_options, OptionKind};

/// `runs` caps how many times a distinct script content may run — `1` (the default) is
/// run-once-per-content; `always` opts out (see `model::exec`). `undo` is deliberately absent:
/// what a removal means is U3, still open, so no key promises it.
/// `undo` is what removing the line runs (U3). Optional, because a script has no inverse and
/// inventing one would be Shall claiming to undo something it cannot: without it, removing an
/// `exec:` drops the record and nothing else, and `plan` says so in those words.
/// `on` names the verb this step belongs to — `sync` (the default), `upgrade`, or `both`.
///
/// **Per step, never inherited (`H6`).** `upgrade` ran no declared steps at all, so a firmware
/// or `rustup` line correctly written and correctly approved was never run by the verb a user
/// reaches for weekly. Widening `upgrade` to run every `exec:` would have made a verb that has
/// never executed user scripts start executing every script in every existing manifest — and
/// the approval gate answers *what* may run, not *which verb* may run it, so somebody who
/// approved a script for `sync` did not thereby approve it for `upgrade`. Writing it on the
/// line makes the widening one step's, and a manifest that says nothing keeps today's meaning.
pub const EXEC_OPTION_KEYS: &[&str] = &["runs", "undo", "on", "after"];

/// Which verbs an `exec:` line belongs to.
///
/// A closed set rather than a comma-separated list, because options are *themselves* separated
/// by commas: `@on=sync,upgrade` parses as `on=sync` plus a second option named `upgrade`,
/// which is `F3`'s boundary confusion invited in by the value grammar. Three names spell the
/// three cases with nothing to disambiguate.
///
/// **The list lives with the type that means it.** This was three strings here and a `Verb`
/// enum in `model::exec`, which is a value the parser accepts and nothing understands the
/// moment the two drift.
pub use crate::model::exec::Verb as ExecVerb;
/// Empty, and stated as a table rather than as a special case in the validator: "what may
/// `generate:` carry" is then answered in the same place as it is for every other kind. It runs
/// every resolution to compute the current answer, so there is no `@runs` ceiling to set.
pub const GENERATE_OPTION_KEYS: &[&str] = &[];

/// `generate:` runs a command and reads declarations from its stdout (U33). It takes no options:
/// unlike `exec:`, it runs every resolution to compute the current answer, so there is no `@runs`
/// ceiling — a ceiling would freeze a stale set. The gate that matters is `allow_generators`
/// (off by default), enforced where the command is actually run, not in the grammar.
pub(super) fn validate_generate(origin: &Origin, name: &str, options: &Options) -> Result<()> {
    if name.trim().is_empty() {
        return Err(
            GrammarError::new(origin.clone(), "`generate:` names no command").with_hint(
                "write `generate:./bin/pick.sh` — a command whose stdout is declarations.",
            ),
        );
    }
    validate_extra_options(
        origin,
        OptionKind::Generate,
        name,
        options,
        Some(
            "a generator runs every resolution to compute the current set, so there is no \
             `@runs` ceiling to set.",
        ),
    )
}

pub(super) fn validate_exec(origin: &Origin, name: &str, options: &Options) -> Result<()> {
    if name.trim().is_empty() {
        return Err(GrammarError::new(origin.clone(), "`exec:` names no script")
            .with_hint("write `exec:./bin/setup.sh` — a path to a script the config carries."));
    }
    validate_extra_options(
        origin,
        OptionKind::Exec,
        name,
        options,
        Some(
            "an exec takes `runs` (a positive number, or `always`), `undo` (a command to run \
             when the line is removed), `on` (`sync`, `upgrade` or `both`), and `after` (how \n             many packages must have moved before it runs).",
        ),
    )?;
    if let Some(on) = options.one("on") {
        let on = on.trim();
        if !ExecVerb::VALUES.contains(&on) {
            return Err(GrammarError::new(
                origin.clone(),
                format!("`exec:{}` has an invalid `on={}`", name, on),
            )
            .with_hint(
                "`on` is `sync` (the default), `upgrade`, or `both`. It says which verb runs \
                 this step; a step `upgrade` should run has to say so, because approving a \
                 script is not the same as approving every verb to run it.",
            ));
        }
    }
    // Which names exist is a fact about the catalogue, so the catalogue answers it — see
    // `model::step::refusal`. Refused here rather than at run time, where "cannot read the
    // script at <config>/step/rustupp" would send a reader looking for a file they never meant
    // to write.
    if let Some((what, hint)) = crate::model::step::refusal(name) {
        return Err(GrammarError::new(origin.clone(), what).with_hint(hint));
    }
    // `@after=N` — run this step only once the run it belongs to has actually moved N
    // packages. Zero is refused rather than read as "always": a threshold of nothing is the
    // absence of a threshold, so writing it means the author expected it to mean something else.
    if let Some(after) = options.one("after") {
        let after = after.trim();
        match after.parse::<usize>() {
            Ok(0) => {
                return Err(GrammarError::new(
                    origin.clone(),
                    format!("`exec:{}` has `after=0`", name),
                )
                .with_hint(
                    "`after` is how many packages must have moved before this step runs, so \n                     `after=0` is the same as leaving it out. Delete it, or write \n                     the number you meant.",
                ));
            }
            Ok(_) => {}
            Err(_) => {
                return Err(GrammarError::new(
                    origin.clone(),
                    format!("`exec:{}` has a non-numeric `after={}`", name, after),
                )
                .with_hint(
                    "`after` is a count of packages, so it is a whole number: `@after=5` runs \n                     the step only on a run that moved at least five.",
                ));
            }
        }
    }
    if let Some(runs) = options.one("runs") {
        let runs = runs.trim();
        if runs != "always" && runs.parse::<u32>().map(|n| n == 0).unwrap_or(true) {
            return Err(GrammarError::new(
                origin.clone(),
                format!("`exec:{}` has an invalid `runs={}`", name, runs),
            )
            .with_hint(
                "`runs` is a positive number (the ceiling on how many times this \
                        content runs) or `always` to run every sync.",
            ));
        }
    }
    Ok(())
}
