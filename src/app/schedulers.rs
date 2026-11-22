//! Reading the schedules this machine already runs (C41).
//!
//! Three dialects, and the rule that governs all of them: **a schedule is offered only when a
//! `schedule:` line can state it exactly.** The line carries a cron expression and a command, so
//! a job whose shape is not a calendar — `@reboot`, launchd's `StartInterval`, a Task Scheduler
//! `EventTrigger`, systemd's `OnUnitActiveSec` — is **refused by name**, carrying the field and
//! the value that make it unexpressible. Widening it to the nearest thing the line can say is
//! worse than refusing: a job that fires at the wrong times reports success, and the person who
//! wrote the original finds out by its consequence (`#195`).
//!
//! The parsers here are pure — text in, findings out — because every one of them is a place
//! where a plausible-looking guess would silently change when a machine does work.

/// One job a dialect reported, in the terms a `schedule:` line uses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    /// A name a `schedule:` line can carry, derived from where the job was found rather than
    /// from its command: two jobs may run the same program, and a name collision would make one
    /// declaration overwrite the other.
    pub name: String,
    pub cron: String,
    pub command: String,
    /// The account the job runs as, when the dialect says so and it is not the invoking one.
    pub user: Option<String>,
    /// Where it was found, for the manifest.
    pub origin: String,
}

/// What reading one dialect produced: the jobs, and the ones it would not state.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Reading {
    pub found: Vec<Found>,
    /// `(name, reason)` — the reason always names the field and the value, never only a verdict.
    pub refused: Vec<(String, String)>,
}

impl Reading {
    fn push_refusal(&mut self, name: impl Into<String>, reason: impl Into<String>) {
        self.refused.push((name.into(), reason.into()));
    }
}

/// The five-field crontab (`/etc/crontab`, `/etc/cron.d/*`, and `crontab -l`).
///
/// `system` selects the format: a system crontab has a **user column** after the five timing
/// fields and a user crontab does not. Reading one as the other is the classic cron bug — it
/// takes the account for the first word of the command, or the command's first word for the
/// account — so the caller says which file it is reading and this function trusts the answer
/// rather than guessing from the shape.
pub fn crontab(text: &str, origin: &str, system: bool, name_prefix: &str) -> Reading {
    let mut out = Reading::default();
    for (index, raw) in text.lines().enumerate() {
        let line = raw.trim();
        let number = index + 1;
        // Blank, a comment, and an environment assignment (`SHELL=/bin/sh`, `PATH = /usr/bin`)
        // are all part of a crontab and none of them is a job. An assignment is `NAME = value`
        // with no whitespace inside the name; the timing fields never contain an `=`.
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if is_assignment(line) {
            continue;
        }

        let fields: Vec<&str> = line.split_whitespace().collect();
        let (cron, rest) = match fields.first() {
            Some(first) if first.starts_with('@') => {
                let macro_name = first.to_string();
                if macro_name == "@reboot" {
                    out.push_refusal(
                        format!("{}-{}", name_prefix, number),
                        format!(
                            "`{}` fires when the machine boots, at {}:{} — a `schedule:` line \
                             carries a calendar (`cron=`), and `@reboot` is not one, so it is \
                             refused rather than widened into a daily job that would run at the \
                             wrong times",
                            macro_name, origin, number
                        ),
                    );
                    continue;
                }
                (macro_name, fields[1..].join(" "))
            }
            Some(_) => {
                // Five timing fields, then either a user and a command (system) or a command.
                let needed = if system { 7 } else { 6 };
                if fields.len() < 5 {
                    out.push_refusal(
                        format!("{}-{}", name_prefix, number),
                        format!(
                            "{}:{} is not five timing fields and a command — a line with {} \
                             field(s) is a syntax this reader will not guess at",
                            origin,
                            number,
                            fields.len()
                        ),
                    );
                    continue;
                }
                if fields.len() < needed {
                    out.push_refusal(
                        format!("{}-{}", name_prefix, number),
                        format!(
                            "{}:{} has no command after its timing fields, so there is nothing \
                             to declare",
                            origin, number
                        ),
                    );
                    continue;
                }
                let cron = fields[..5].join(" ");
                let rest = fields[5..].join(" ");
                if crate::model::schedule::validate_cron(&cron).is_err() {
                    out.push_refusal(
                        format!("{}-{}", name_prefix, number),
                        format!(
                            "`{}` at {}:{} is not a cron expression Shall can carry",
                            cron, origin, number
                        ),
                    );
                    continue;
                }
                (cron, rest)
            }
            None => continue,
        };

        let (user, command) = if system {
            let mut words = rest.splitn(2, char::is_whitespace);
            let user = words.next().unwrap_or("").trim().to_string();
            let command = words.next().unwrap_or("").trim().to_string();
            (Some(user), command)
        } else {
            (None, rest.trim().to_string())
        };
        if command.is_empty() || user.as_deref() == Some("") {
            out.push_refusal(
                format!("{}-{}", name_prefix, number),
                format!(
                    "{}:{} has no command after its timing fields, so there is nothing to declare",
                    origin, number
                ),
            );
            continue;
        }
        out.found.push(Found {
            name: format!("{}-{}", name_prefix, number),
            cron,
            command,
            user,
            origin: format!("{}:{}", origin, number),
        });
    }
    out
}

/// A systemd `OnCalendar=` value, as a cron expression — or `None` when it is not a calendar a
/// five-field line can state.
///
/// **The mapping is closed, not a translation.** Only the spellings that mean exactly what the
/// cron expression means are accepted: the named intervals, and `[-*-*] HH:MM[:SS]` with an
/// optional weekday list. Everything else — a `/step`, a seconds field that is not `00`, a
/// comma-separated list of calendar specs, a `~` last-day-of-month — is refused, because the
/// nearest cron expression differs from it and the difference would be invisible.
pub fn on_calendar_to_cron(spec: &str) -> Option<String> {
    let spec = spec.trim();
    let named = match spec.to_ascii_lowercase().as_str() {
        "hourly" => Some("0 * * * *"),
        "daily" | "midnight" => Some("0 0 * * *"),
        "weekly" => Some("0 0 * * 1"),
        "monthly" => Some("0 0 1 * *"),
        "yearly" | "annually" => Some("0 0 1 1 *"),
        _ => None,
    };
    if let Some(cron) = named {
        return Some(cron.to_string());
    }
    // A step (`*:0/15`) and a floating day (`~`, the last day of the month) have no cron
    // spelling at all. Refused here rather than rounded to the expression that looks closest.
    if spec.contains('/') || spec.contains('~') {
        return None;
    }
    // `HH:MM`, `*-*-* HH:MM` and `Mon *-*-* HH:MM` are the three shapes a calendar takes; the
    // weekday may precede the date part, and the date part must be the every-day wildcard.
    let (days, time) = match spec.rsplit_once(' ') {
        Some((days, time)) => (Some(days), time),
        None => (None, spec),
    };
    let days = match days {
        None => None,
        Some(days) => {
            let words: Vec<&str> = days.split_whitespace().collect();
            match words.as_slice() {
                [date] => {
                    if *date == "*-*-*" {
                        None
                    } else {
                        Some(*date)
                    }
                }
                [weekdays, date] if *date == "*-*-*" => Some(*weekdays),
                _ => return None,
            }
        }
    };
    // A second `HH:MM:SS` is accepted only when `SS` is `00`: cron has no seconds field, and
    // `09:30:15` would become a job that runs at 09:30.
    // Two times of day (`09:00,17:00`) is one systemd line and two cron jobs. A comma in the
    // time part is refused; a comma in the day part is a weekday list, which cron does say.
    if time.contains(',') {
        return None;
    }
    let mut parts = time.split(':');
    let hour = parts.next()?;
    let minute = parts.next()?;
    match parts.next() {
        None => {}
        Some("00") => {}
        Some(_) => return None,
    }
    if parts.next().is_some() {
        return None;
    }
    // The date part is the every-day wildcard, because that is the only calendar this function
    // accepts: a spec carrying a month or a day-of-month (`*-03-01 04:00`) falls through the
    // `HH:MM` shape below and is refused, so a cron expression never claims a date the spec did
    // not say.
    let day_of_month = "*";
    let month = "*";
    // The date part is either the every-day wildcard or a weekday list. Anything that names a
    // month or a day of the month (`*-*-03`) is refused: a cron expression with `* *` in those
    // fields would run on every day of the month instead, which is a different job.
    let dow = match days {
        None => "*".to_string(),
        // A weekday list is the only thing left; a date part naming a month or a day of month
        // (`* *-03-01`) reached the shape check above and was refused there.
        Some(days) if days.contains('-') && !days.contains("..") => return None,
        Some(days) => days_to_cron(days)?,
    };
    // `*:00:00` is "every hour on the hour", which cron writes as `0 * * * *`: an asterisk is a
    // value in both dialects, and a numeric parse that rejected it turned this host's own
    // `logrotate` timer into a refusal.
    let minute = number_or_star(minute)?;
    let hour = number_or_star(hour)?;
    Some(format!(
        "{} {} {} {} {}",
        minute, hour, day_of_month, month, dow
    ))
}

/// Whether a crontab line is an environment assignment (`SHELL=/bin/sh`, `PATH = /usr/bin`)
/// rather than a job.
///
/// The left side of the `=` must be a bare name with no whitespace: a command line can contain
/// `=` anywhere (`--opt=value`), and reading one of those as an assignment would drop a job.
fn is_assignment(line: &str) -> bool {
    let Some((left, _)) = line.split_once('=') else {
        return false;
    };
    let left = left.trim();
    !left.is_empty()
        && !left.contains(char::is_whitespace)
        && left
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && left.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// A time field: a number, or `*` for "every". Anything else (`3,15`, `*/2`, `1-5`) is a shape
/// this mapping does not state, because cron's own spelling of it differs in step semantics.
fn number_or_star(value: &str) -> Option<String> {
    if value == "*" {
        return Some("*".to_string());
    }
    let number: u8 = value.parse().ok()?;
    Some(number.to_string())
}

/// systemd's weekday spelling, as cron's day-of-week field. Ranges (`Mon..Fri`), lists
/// (`Mon,Wed`) and single days are the three shapes; anything else is not this field.
fn days_to_cron(days: &str) -> Option<String> {
    fn one(day: &str) -> Option<u8> {
        Some(match day.to_ascii_lowercase().as_str() {
            "mon" => 1,
            "tue" => 2,
            "wed" => 3,
            "thu" => 4,
            "fri" => 5,
            "sat" => 6,
            // 7, not 0: the cron parser this tree uses refuses `0` in the day-of-week field
            // ("Days of Week must be greater than or equal to 1"), so a Sunday mapped to `0`
            // produced a schedules file that did not parse — found by CI on a machine whose
            // `e2scrub_all` timer runs on Sunday.
            "sun" => 7,
            _ => return None,
        })
    }
    if let Some((from, to)) = days.split_once("..") {
        return Some(format!("{}-{}", one(from)?, one(to)?));
    }
    let mut out = Vec::new();
    for day in days.split(',') {
        out.push(one(day)?.to_string());
    }
    Some(out.join(","))
}

/// What `systemctl list-timers --all --no-pager` printed, as timer unit names.
///
/// The columns are `NEXT LEFT LAST PASSED UNIT ACTIVATES`, and a timer that has never fired
/// prints `n/a` in the first four — so the unit is found by shape (`◾ ◾ ◾ ◾ NAME ACTIVATES`)
/// with the unit being the first column that ends in `.timer`. Reading the last-but-one column
/// instead would drop every timer not yet triggered, which is most of them on a fresh machine.
pub fn timers_in_listing(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in text.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(unit) = line
            .split_whitespace()
            .find(|word| word.ends_with(".timer"))
        {
            if !out.iter().any(|u| u == unit) {
                out.push(unit.to_string());
            }
        }
    }
    out
}

/// What a timer's calendar turned out to be, from `systemctl show -p OnCalendar -p TimersCalendar
/// -p TimersMonotonic`.
///
/// **Three answers, not two.** A timer with no calendar at all (it is driven by an interval) and
/// a timer whose property this systemd does not print are different findings, and reporting the
/// second as the first tells the person their `OnCalendar=hourly` unit is an interval — which is
/// exactly what happened when this reader asked only for `-p OnCalendar` and newer systemd
/// answered with nothing at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Calendar {
    /// A calendar the cron mapping can try to state.
    Spec(String),
    /// `OnBootSec`, `OnUnitActiveSec`, `OnActiveSec`: an interval, not a calendar.
    Interval(String),
    /// Neither — the property was not printed. Not a guess: it is reported as unreadable so the
    /// caller can say so rather than claim the timer has no calendar.
    Unreadable,
}

/// Parse that `systemctl show` output into a [`Calendar`].
pub fn calendar_of(properties: &str) -> Calendar {
    // The plain property, on systemd versions that print it.
    if let Some(spec) = properties
        .lines()
        .find_map(|line| line.strip_prefix("OnCalendar="))
        .map(str::trim)
        .filter(|value| !value.is_empty() && *value != "n/a")
    {
        return Calendar::Spec(spec.to_string());
    }
    // `TimersCalendar={ OnCalendar=*-*-* *:00:00 ; next_elapse=… }` — the structured form a
    // current systemd prints instead, with the recurrence inside the braces.
    if let Some(spec) = properties
        .lines()
        .find_map(|line| line.strip_prefix("TimersCalendar="))
        .and_then(|value| value.split("OnCalendar=").nth(1))
        .and_then(|rest| rest.split(';').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return Calendar::Spec(spec.to_string());
    }
    // An interval. Named so the refusal can quote the field that stopped it. A zero is not an
    // interval: `TimersMonotonic` prints unset fields as `OnBootSec=0s`, and reporting one of
    // those would turn "we could not read a calendar" into a confident wrong answer.
    // `TimersMonotonic` prints the microsecond spelling (`OnBootUSec=15min`), so the field is
    // read by shape and reported under the name the unit file uses (`OnBootSec`).
    for stem in ["OnBoot", "OnUnitActive", "OnActive", "OnStartup"] {
        for spelling in [format!("{}Sec=", stem), format!("{}USec=", stem)] {
            let Some(rest) = properties.split(&spelling).nth(1) else {
                continue;
            };
            let value = rest.split([';', ' ', '\n']).next().unwrap_or("");
            let number = value.trim_end_matches(|c: char| c.is_ascii_alphabetic());
            if number.is_empty() || number.parse::<f64>() == Ok(0.0) {
                continue;
            }
            return Calendar::Interval(format!("{}Sec", stem));
        }
    }
    Calendar::Unreadable
}

/// A name a `schedule:` line can carry, from a unit or file name.
///
/// `shall-nightly.timer` and `shall-nightly` must not both become `shall-nightly` twice under
/// different dialects, so the dialect stays in the name; everything outside the grammar's name
/// characters becomes a `-` rather than being dropped, so two different names cannot collapse
/// into one.
pub fn name_from(prefix: &str, raw: &str) -> String {
    let squashed: String = raw
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect();
    format!("{}-{}", prefix, squashed.trim_matches('-'))
}
