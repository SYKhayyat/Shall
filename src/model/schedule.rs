//! Turning a `schedule:` line into a provisionable task (S21).
//!
//! A `schedule:` line names a job and carries its options — when to run and what to run:
//!
//! ```text
//! schedule:nightly-tidy@cron=0 2 * * *,run=clean,notify=desktop
//!
//! schedule:nightly-tidy {
//!   cron = 0 2 * * *
//!   run = clean
//!   notify = desktop
//!   enabled = true       # provision it and leave it silent with `false`
//!   persistent = true    # run a firing the machine was switched off for
//!   jitter = 30m         # spread a fleet out around the scheduled moment
//!   elevated = false     # run with the highest privileges the account holds
//! }
//! ```
//!
//! The four options past `notify` are **not** universal, and this module does not pretend
//! otherwise: it parses and bounds them, and each provisioner in `app/scheduler` either
//! expresses the option or refuses it by name. Accepting an option and dropping it is the same
//! failure as a cron silently widened into DAILY — the declaration says one thing, the machine
//! does another, and both report success.
//!
//! The resolver collects these (from the `schedules` file only — II.2), and this module maps
//! one to the [`ScheduleConfig`] the existing `SchedulerManager` provisions onto systemd /
//! launchd / Task Scheduler. It is pure: no I/O, no provisioning — just the line-to-config
//! translation and the validation that a job a machine will run on a timer is fully specified.

use crate::config::config::ScheduleConfig;
use crate::config::grammar::{GrammarError, Options, Origin, Result};

/// The keys a `schedule:` line understands live with the rest of II.2's option tables, in the
/// grammar. `cron` and `run` are required — a timed job with no schedule or no command is not
/// a job — and `notify` is optional.
use crate::config::grammar::statement::SCHEDULE_OPTION_KEYS as KNOWN_KEYS;

/// Build a [`ScheduleConfig`] from a `schedule:<name>` line's options, or an error that names
/// the file and line and says exactly what is missing or unrecognized.
pub fn schedule_config(
    name: &str,
    options: &Options,
    origin: &Origin,
    never_unattended: &[String],
) -> Result<ScheduleConfig> {
    // Unknown keys are an error, not ignored: a typo'd `crron =` would otherwise leave the job
    // with no schedule and no complaint, which is the class of silent failure II.2 refuses.
    for key in options.keys() {
        if !KNOWN_KEYS.contains(&key) {
            return Err(GrammarError::new(
                origin.clone(),
                format!("`schedule:{}` has an unknown option `{}`", name, key),
            )
            .with_hint(format!(
                "a schedule takes {}.",
                KNOWN_KEYS
                    .iter()
                    .map(|k| format!("`{}`", k))
                    .collect::<Vec<_>>()
                    .join(", ")
            )));
        }
    }

    let cron = required(name, options, "cron", origin)?;
    validate_cron(&cron).map_err(|e| {
        GrammarError::new(
            origin.clone(),
            format!("`schedule:{}` has an invalid cron: {}", name, e),
        )
        .with_hint(
            "five fields (min hour dom month dow), e.g. `0 2 * * *`, or a macro like `@daily`.",
        )
    })?;
    let command = required(name, options, "run", origin)?;
    refuse_unattended(name, &command, origin, never_unattended)?;
    let notification = options.one("notify").map(str::to_string);

    Ok(ScheduleConfig {
        name: name.to_string(),
        cron,
        command,
        notification,
        enabled: boolean(name, options, "enabled", origin)?,
        persistent: boolean(name, options, "persistent", origin)?,
        jitter: seconds(name, options, "jitter", origin)?,
        elevated: boolean(name, options, "elevated", origin)?,
        user: options.one("user").map(str::to_string),
    })
}

/// A yes/no option, or an error naming the line.
///
/// **Anything that is not a spelling of true or false is refused**, rather than read as false
/// the way `service:`'s `enabled` reads it. A typo that silently means "no" on an option whose
/// job is to stop a job firing is the quiet failure this file's unknown-key check exists to
/// prevent, one level down.
fn boolean(name: &str, options: &Options, key: &str, origin: &Origin) -> Result<Option<bool>> {
    let Some(raw) = options.one(key) else {
        return Ok(None);
    };
    match raw.trim().to_ascii_lowercase().as_str() {
        "true" | "yes" | "on" | "1" => Ok(Some(true)),
        "false" | "no" | "off" | "0" => Ok(Some(false)),
        other => Err(GrammarError::new(
            origin.clone(),
            format!(
                "`schedule:{}` has `{} = {}`, which is not yes or no",
                name, key, other
            ),
        )
        .with_hint("write `true` or `false` (`yes`/`no`, `on`/`off` and `1`/`0` also read).")),
    }
}

/// The longest jitter that is still a jitter. A day of randomised delay on a daily timer means
/// the firing can land after the next one is due, which is not a spread — it is a schedule
/// nobody can predict.
const MAX_JITTER_SECONDS: u32 = 24 * 60 * 60;

/// A duration in seconds, written bare or with a `s`/`m`/`h` suffix.
fn seconds(name: &str, options: &Options, key: &str, origin: &Origin) -> Result<Option<u32>> {
    let Some(raw) = options.one(key) else {
        return Ok(None);
    };
    let raw = raw.trim();
    let bad = |what: &str| {
        Err(GrammarError::new(
            origin.clone(),
            format!(
                "`schedule:{}` has `{} = {}`, which {}",
                name, key, raw, what
            ),
        )
        .with_hint(format!(
            "write a duration: `900`, `15m`, `2h`. The longest this option takes is {}h.",
            MAX_JITTER_SECONDS / 3600
        )))
    };
    let (digits, multiplier) = match raw.chars().last() {
        Some('s') | Some('S') => (&raw[..raw.len() - 1], 1),
        Some('m') | Some('M') => (&raw[..raw.len() - 1], 60),
        Some('h') | Some('H') => (&raw[..raw.len() - 1], 3600),
        _ => (raw, 1),
    };
    let Ok(value) = digits.trim().parse::<u32>() else {
        return bad("is not a duration");
    };
    let Some(total) = value.checked_mul(multiplier) else {
        return bad("is longer than any schedule");
    };
    if total > MAX_JITTER_SECONDS {
        return bad("is longer than a day");
    }
    Ok(Some(total))
}

/// Refuse a `run` whose command is on this machine's `[guard] never_unattended` list (K13).
///
/// The list arrives as an argument rather than being read here so the rule stays one fact in
/// one place — `preferences.toml` — and so the check is testable without a config on disk.
fn refuse_unattended(
    name: &str,
    command: &str,
    origin: &Origin,
    never_unattended: &[String],
) -> Result<()> {
    let head = command.split_whitespace().next().unwrap_or("");
    if never_unattended.iter().any(|c| c == head) {
        return Err(GrammarError::new(
            origin.clone(),
            format!("`schedule:{}` may not run `{}`", name, head),
        )
        .with_hint(format!(
            "a schedule runs unattended, and this command removes software. `{}` is in \
             `[guard] never_unattended` in preferences.toml, which currently reads [{}]; take \
             it out of that list to permit it, or run the command yourself.",
            head,
            never_unattended.join(", ")
        )));
    }
    Ok(())
}

fn required(name: &str, options: &Options, key: &str, origin: &Origin) -> Result<String> {
    match options.one(key) {
        Some(v) if !v.trim().is_empty() => Ok(v.to_string()),
        _ => Err(GrammarError::new(
            origin.clone(),
            format!("`schedule:{}` is missing `{}`", name, key),
        )
        .with_hint(match key {
            "cron" => "add `cron = <expression>`, e.g. `cron = 0 2 * * *` for 2am daily.",
            "run" => "add `run = <command>`, e.g. `run = clean`.",
            _ => "a required option is missing.",
        })),
    }
}

/// Is this a cron expression the OS scheduler will accept?
///
/// Standard cron is 5-field (min hour dom month dow) and the `cron` crate wants 6 with
/// seconds, so a 5-field expression is normalized by prepending `0`. `@`-macros never reach
/// the parser — the systemd/launchd mapping handles those.
///
/// One implementation, called at parse time so the error can name the file and line, and
/// again by the provisioner for the config it is handed.
pub fn validate_cron(cron: &str) -> std::result::Result<(), String> {
    if cron.starts_with('@') {
        return Ok(());
    }
    let normalized = if cron.split_whitespace().count() == 5 {
        format!("0 {}", cron)
    } else {
        cron.to_string()
    };
    normalized
        .parse::<cron::Schedule>()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Write a `schedule:` block into the `schedules` file body.
///
/// `schedule add` is a shortcut for editing this file, the way `install` is a shortcut for
/// editing a module (P1): the file is the state, so the edit IS the command and `sync`
/// provisions it. There is no second store — a `[schedules]` table in preferences was one,
/// and the two could disagree about what this machine runs.
pub fn add_line(
    body: &str,
    name: &str,
    cron: &str,
    run: &str,
    notify: Option<&str>,
) -> std::result::Result<String, String> {
    if find_block(body, name).is_some() {
        return Err(format!(
            "`schedule:{}` is already in the `schedules` file. Remove it first, or edit the \
             file — two lines with one name is a schedule that means whichever the reader \
             saw last.",
            name
        ));
    }

    let mut out = body.to_string();
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    if !out.is_empty() {
        out.push('\n');
    }
    out.push_str(&format!("schedule:{} {{\n", name));
    out.push_str(&format!("  cron = {}\n", cron));
    out.push_str(&format!("  run  = {}\n", run));
    if let Some(n) = notify {
        out.push_str(&format!("  notify = {}\n", n));
    }
    out.push_str("}\n");
    Ok(out)
}

/// Take a `schedule:` line or block out of the body. `None` if no such name is in the file.
pub fn remove_line(body: &str, name: &str) -> Option<String> {
    let (start, end) = find_block(body, name)?;
    let kept: Vec<&str> = body
        .lines()
        .enumerate()
        .filter(|(i, _)| *i < start || *i > end)
        .map(|(_, l)| l)
        .collect();
    // Rejoin with the ending the file already used — a bare `\n` turned every CRLF schedules
    // file into an LF one in full (see `edit.rs::rejoin` for why that is a whole-file diff).
    let eol = if body.contains("\r\n") { "\r\n" } else { "\n" };
    let mut out = kept.join(eol);
    if !out.is_empty() {
        out.push_str(eol);
    }
    Some(out)
}

/// The 0-based line range a `schedule:NAME` occupies — one line in short form, header
/// through `}` in block form.
fn find_block(body: &str, name: &str) -> Option<(usize, usize)> {
    let lines: Vec<&str> = body.lines().collect();
    let header = format!("schedule:{}", name);
    for (i, raw) in lines.iter().enumerate() {
        let text = crate::config::grammar::strip_comment(raw).trim();
        // `schedule:nightly` must not match `schedule:nightly-tidy`, so the name has to end
        // where the header does or be followed by an option, a brace or space.
        let Some(rest) = text.strip_prefix(&header) else {
            continue;
        };
        if rest.starts_with(|c: char| c.is_alphanumeric() || c == '-' || c == '_') {
            continue;
        }
        if crate::config::grammar::block_header(rest).is_none() {
            return Some((i, i));
        }
        for (j, close) in lines.iter().enumerate().skip(i + 1) {
            if close.trim() == "}" {
                return Some((i, j));
            }
        }
        return Some((i, lines.len().saturating_sub(1)));
    }
    None
}
