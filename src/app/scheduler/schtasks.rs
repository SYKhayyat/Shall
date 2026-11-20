//! The scheduling dialect of Task Scheduler: XML, triggers, and what `schtasks` can say.

use super::{
    armed, unescape_xml, xml_element, HeldTask, Provisioned, Reading, TaskProvisioner, TaskTarget,
};
use crate::config::config::ScheduleConfig;
use crate::core::{CommandExecutor, Error, Result};
use async_trait::async_trait;
use std::path::Path;

/// The five cron fields, with `@`-shorthands already expanded into them.
///
/// One expansion, shared. The shorthand table was written out once per provisioner, and the
/// Windows one simply did not have it — so `@daily` reached `split_whitespace()` as a single
/// field and came out the other side as the start time `02:@daily`. A table each is a table
/// that can be missing.
struct CronFields<'a> {
    minute: &'a str,
    hour: &'a str,
    dom: &'a str,
    month: &'a str,
    dow: &'a str,
}

/// `@weekly` is Monday here, not Sunday, because that is what the systemd and launchd mappings
/// have always done (`OnCalendar=weekly` is Mon 00:00). Matching vixie-cron instead would move
/// existing users' schedules by a day on two platforms to fix a third.
fn parse_cron(cron: &str) -> Option<CronFields<'_>> {
    let f = |minute, hour, dom, month, dow| {
        Some(CronFields {
            minute,
            hour,
            dom,
            month,
            dow,
        })
    };
    match cron.trim() {
        "@hourly" => f("0", "*", "*", "*", "*"),
        "@daily" | "@midnight" => f("0", "0", "*", "*", "*"),
        "@weekly" => f("0", "0", "*", "*", "1"),
        "@monthly" => f("0", "0", "1", "*", "*"),
        "@yearly" | "@annually" => f("0", "0", "1", "1", "*"),
        other => {
            let p: Vec<&str> = other.split_whitespace().collect();
            if p.len() < 5 {
                return None;
            }
            f(p[0], p[1], p[2], p[3], p[4])
        }
    }
}

/// `HH:mm`, which is the only start time Task Scheduler accepts.
///
/// The whole reported defect was `format!("{}:{}", hour, min)` on the raw cron fields: `0 3 * * *`
/// became `3:0` and `schtasks` answered `ERROR: Invalid starttime value.` A time is two digits
/// and two digits, always.
fn schtasks_time(hour: &str, minute: &str) -> Option<String> {
    let h: u8 = hour.parse().ok()?;
    let m: u8 = minute.parse().ok()?;
    if h > 23 || m > 59 {
        return None;
    }
    Some(format!("{:02}:{:02}", h, m))
}

/// Sunday-first day names, the order Task Scheduler's `<DaysOfWeek>` lists them in.
const DAY_NAMES: [&str; 7] = ["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"];
/// The `<DaysOfWeek>` element names, in the same order.
const DAY_ELEMENTS: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];
const MONTH_NAMES: [&str; 12] = [
    "JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC",
];
/// The `<Months>` element names, in the same order.
const MONTH_ELEMENTS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// cron's day-of-week (0/7 = Sunday) as the day names `/D` takes, comma-separated.
fn schtasks_days(dow: &str) -> Option<String> {
    let name = |n: u8| DAY_NAMES.get((n % 7) as usize).copied();

    let mut out: Vec<&str> = Vec::new();
    for part in dow.split(',') {
        // `/D` takes a list and no ranges, so a range is expanded rather than passed through.
        if let Some((a, b)) = part.split_once('-') {
            let (a, b): (u8, u8) = (a.parse().ok()?, b.parse().ok()?);
            if a > b || b > 7 {
                return None;
            }
            for n in a..=b {
                out.push(name(n)?);
            }
        } else {
            out.push(name(part.parse().ok()?)?);
        }
    }
    if out.is_empty() {
        return None;
    }
    out.dedup();
    Some(out.join(","))
}

/// The `/SC …` arguments that make Task Scheduler do what a cron line says.
///
/// Windows has no cron, so this is a translation and not every sentence has one. Where a cron
/// cannot be expressed it is **refused by name** rather than widened into the nearest thing
/// Task Scheduler can do: the defect this replaces silently turned `0 3 * * 1` into a DAILY
/// task, which ran seven times as often as it was declared to and reported success each time.
/// A schedule that fires when it should not is worse than one that refuses to be created.
/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
/// crate links the library without `cfg(test)`, so a private item is invisible there.
pub fn map_cron_to_schtasks(cron: &str) -> std::result::Result<Vec<String>, String> {
    let s = |v: &str| v.to_string();
    let cannot = || {
        format!(
            "`{}` is a schedule Windows Task Scheduler cannot express. It understands a time of \
             day, a weekday, a day of the month, or a fixed interval — but not a combination of \
             an interval with a time window. Split it into separate schedules, or use a cron \
             this machine can keep.",
            cron
        )
    };

    if cron.trim() == "@reboot" {
        return Ok(vec![s("/SC"), s("ONSTART")]);
    }

    let c = parse_cron(cron).ok_or_else(|| {
        format!(
            "`{}` is not a cron expression: it needs five fields (minute hour day month weekday) \
             or one of @reboot, @hourly, @daily, @weekly, @monthly, @yearly.",
            cron
        )
    })?;

    // Sub-hourly. Task Scheduler's MINUTE/HOURLY intervals run around the clock, so they can
    // carry no other constraint — and pretending otherwise is how a schedule fires at times
    // nobody asked for.
    let unconstrained = c.dom == "*" && c.month == "*" && c.dow == "*";
    if c.minute == "*" || c.minute.starts_with("*/") {
        if !unconstrained || c.hour != "*" {
            return Err(cannot());
        }
        let mut args = vec![s("/SC"), s("MINUTE")];
        if let Some(step) = c.minute.strip_prefix("*/") {
            step.parse::<u16>().map_err(|_| cannot())?;
            args.extend([s("/MO"), s(step)]);
        }
        // Anchor the repetition: without /ST Task Scheduler starts counting from
        // registration time, invisible to the canonical comparison and to the user.
        let st = schtasks_time("0", "0").unwrap();
        args.extend([s("/ST"), st]);
        return Ok(args);
    }
    if c.hour == "*" || c.hour.starts_with("*/") {
        if !unconstrained {
            return Err(cannot());
        }
        let mut args = vec![s("/SC"), s("HOURLY")];
        if let Some(step) = c.hour.strip_prefix("*/") {
            step.parse::<u16>().map_err(|_| cannot())?;
            args.extend([s("/MO"), s(step)]);
        }
        // The first run of the hour, which is what the minute field means here.
        let st = schtasks_time("0", c.minute).ok_or_else(cannot)?;
        args.extend([s("/ST"), st]);
        return Ok(args);
    }

    let st = schtasks_time(c.hour, c.minute).ok_or_else(cannot)?;

    // A weekday beats a day of the month: `/SC WEEKLY` and `/SC MONTHLY` are exclusive, and a
    // cron naming both is the one shape with no Task Scheduler equivalent.
    if c.dow != "*" {
        if c.dom != "*" {
            return Err(cannot());
        }
        let days = schtasks_days(c.dow).ok_or_else(cannot)?;
        return Ok(vec![s("/SC"), s("WEEKLY"), s("/D"), days, s("/ST"), st]);
    }

    if c.dom != "*" || c.month != "*" {
        let mut args = vec![s("/SC"), s("MONTHLY")];
        if c.month != "*" {
            let n: usize = c.month.parse().map_err(|_| cannot())?;
            let name = MONTH_NAMES.get(n.wrapping_sub(1)).ok_or_else(cannot)?;
            args.extend([s("/M"), s(name)]);
        }
        // `/SC MONTHLY` with no `/D` is the 1st, which is also what a bare month means.
        let day = if c.dom == "*" { "1" } else { c.dom };
        day.parse::<u8>()
            .ok()
            .filter(|d| (1..=31).contains(d))
            .ok_or_else(cannot)?;
        args.extend([s("/D"), s(day), s("/ST"), st]);
        return Ok(args);
    }

    Ok(vec![s("/SC"), s("DAILY"), s("/ST"), st])
}

/// The value after `flag` in a `/SC …` argument list.
fn arg_after<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    let i = args.iter().position(|a| a == flag)?;
    args.get(i + 1).map(String::as_str)
}

/// Day names in Sunday-first order, so a cron that lists them in another order compares equal
/// to the same set read back out of `<DaysOfWeek>`. `5,1` and `1,5` are one schedule.
fn sorted_days(list: &str) -> String {
    let mut days: Vec<&str> = DAY_NAMES
        .iter()
        .copied()
        .filter(|d| list.split(',').any(|part| part.trim() == *d))
        .collect();
    days.dedup();
    days.join(",")
}

/// When a task fires, in one spelling both sides of the comparison can produce.
///
/// Task Scheduler keeps no file to diff, so the declaration is canonicalised from the `/SC`
/// arguments Shall would pass and the machine's copy is canonicalised from the trigger XML it
/// hands back. Anything neither can express is not canonicalised into something close — the
/// reader returns `None` and the reading is *unreadable*, which is the V.188 answer: a shape
/// Shall does not understand must never be reported as drift, or it re-writes the task on every
/// sync for ever.
/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
/// crate links the library without `cfg(test)`, so a private item is invisible there.
pub fn schtasks_when(args: &[String]) -> Option<String> {
    let sc = arg_after(args, "/SC")?;
    let every = arg_after(args, "/MO").unwrap_or("1");
    let at = arg_after(args, "/ST").unwrap_or("00:00");
    Some(match sc {
        "ONSTART" => "boot".to_string(),
        "MINUTE" => format!("every {}m", every),
        "HOURLY" => format!("every {}h from {}", every, at),
        "DAILY" => format!("daily at {}", at),
        "WEEKLY" => format!(
            "weekly on {} at {}",
            sorted_days(arg_after(args, "/D")?),
            at
        ),
        "MONTHLY" => {
            let day = arg_after(args, "/D").unwrap_or("1");
            match arg_after(args, "/M") {
                Some(month) => format!("monthly in {} on {} at {}", month, day, at),
                None => format!("monthly on {} at {}", day, at),
            }
        }
        _ => return None,
    })
}

/// The text between `<tag>` and `</tag>`, first occurrence, un-escaped for the five XML
/// entities Task Scheduler emits. Deliberately not a parser: the document is machine-written,
/// the elements wanted are leaves, and a dependency for four `find` calls is a dependency.
/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
/// crate links the library without `cfg(test)`, so a private item is invisible there.
pub fn xml_text<'a>(doc: &'a str, tag: &str) -> Option<&'a str> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let start = doc.find(&open)? + open.len();
    let end = doc[start..].find(&close)? + start;
    Some(doc[start..end].trim())
}

/// How many trigger elements a `<Triggers>` block opens, whatever kind they are.
///
/// Any element whose name ends in `Trigger` counts — `<Triggers>` itself does not, and neither
/// does a closing tag. Shall writes exactly one.
/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
/// crate links the library without `cfg(test)`, so a private item is invisible there.
pub fn trigger_count(triggers: &str) -> usize {
    triggers
        .match_indices('<')
        .filter(|(at, _)| {
            let rest = &triggers[at + 1..];
            !rest.starts_with('/')
                && rest
                    .split(['>', ' ', '/'])
                    .next()
                    .is_some_and(|name| name.ends_with("Trigger"))
        })
        .count()
}

/// `PT15M` / `PT6H` as the interval Shall spells `every 15m`.
///
/// Day-scale repetitions (`P1D`) deliberately have no reading: Shall writes a daily schedule as
/// a `ScheduleByDay` calendar trigger, never as a repetition, so a `P1D` under Shall's task name
/// is somebody else's task and describing it would be a guess.
fn iso_interval(spec: &str) -> Option<String> {
    let body = spec.strip_prefix('P')?;
    let time = body.strip_prefix('T')?;
    if let Some(m) = time.strip_suffix('M') {
        m.parse::<u32>().ok()?;
        return Some(format!("every {}m", m));
    }
    let h = time.strip_suffix('H')?;
    h.parse::<u32>().ok()?;
    Some(format!("every {}h", h))
}

/// `2010-12-16T03:00:00` as `03:00`.
fn boundary_time(boundary: &str) -> Option<String> {
    let time = boundary.split('T').nth(1)?;
    let (h, rest) = time.split_once(':')?;
    let m = rest.split(':').next()?;
    schtasks_time(h, m)
}

/// The canonical `when` for the single trigger in a task's XML, or `None` for any shape Shall
/// does not write and therefore cannot compare.
/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
/// crate links the library without `cfg(test)`, so a private item is invisible there.
pub fn when_from_xml(doc: &str) -> Option<String> {
    let triggers = xml_element(doc, "Triggers")?;
    // Shall writes exactly one trigger. More than one means somebody else has been here, and
    // describing what they did is not something this reader can do honestly.
    //
    // **Counted by shape, not against a list of the kinds this reader understands.** A list
    // would count the calendar trigger, miss the `<EventTrigger>` beside it, and report the
    // task as matching a declaration that says nothing about the event — which is the reader
    // claiming to have read something it never saw.
    if trigger_count(triggers) != 1 {
        return None;
    }

    if triggers.contains("<BootTrigger") {
        return Some("boot".to_string());
    }

    let at = xml_text(triggers, "StartBoundary").and_then(boundary_time);

    // A repetition is the only place Task Scheduler can put a sub-daily interval, whichever
    // trigger element carries it.
    if let Some(repetition) = xml_element(triggers, "Repetition") {
        let interval = iso_interval(xml_text(repetition, "Interval")?)?;
        return Some(match interval.strip_suffix(|c: char| c == 'h') {
            Some(_) => format!("{} from {}", interval, at?),
            None => interval,
        });
    }

    if let Some(daily) = xml_element(triggers, "ScheduleByDay") {
        // Shall only ever writes a one-day interval; anything else is not its task.
        if xml_text(daily, "DaysInterval").unwrap_or("1") != "1" {
            return None;
        }
        return Some(format!("daily at {}", at?));
    }

    if let Some(weekly) = xml_element(triggers, "ScheduleByWeek") {
        if xml_text(weekly, "WeeksInterval").unwrap_or("1") != "1" {
            return None;
        }
        let days = xml_element(weekly, "DaysOfWeek")?;
        let named: Vec<&str> = DAY_ELEMENTS
            .iter()
            .enumerate()
            .filter(|(_, element)| {
                days.contains(&format!("<{} ", element))
                    || days.contains(&format!("<{}/", element))
                    || days.contains(&format!("<{}>", element))
            })
            .map(|(i, _)| DAY_NAMES[i])
            .collect();
        if named.is_empty() {
            return None;
        }
        return Some(format!("weekly on {} at {}", named.join(","), at?));
    }

    if let Some(monthly) = xml_element(triggers, "ScheduleByMonth") {
        let day = xml_text(xml_element(monthly, "DaysOfMonth")?, "Day")?.to_string();
        let month = xml_element(monthly, "Months").and_then(|months| {
            MONTH_ELEMENTS
                .iter()
                .enumerate()
                .find(|(_, element)| {
                    months.contains(&format!("<{} ", element))
                        || months.contains(&format!("<{}/", element))
                        || months.contains(&format!("<{}>", element))
                })
                .map(|(i, _)| MONTH_NAMES[i])
        });
        return Some(match month {
            Some(m) => format!("monthly in {} on {} at {}", m, day, at?),
            None => format!("monthly on {} at {}", day, at?),
        });
    }

    None
}

/// The command a task's `/XML` document registers, in the unquoted form Task Scheduler stores
/// it — `program` with its quotes trimmed, then the arguments verbatim.
///
/// One derivation, because three callers need it: `provisioned_from_xml` compares it against
/// the rendered spec, the sweep's read-back of who declared a task asks it for the declaring
/// run's identity (#182), and a windows-only test pins it.
/// **`pub` for the tests' sake**, like its sibling above.
pub fn command_from_xml(doc: &str) -> Option<String> {
    let exec = xml_element(doc, "Exec")?;
    let program = unescape_xml(xml_text(exec, "Command")?);
    let arguments = xml_text(exec, "Arguments")
        .map(unescape_xml)
        .unwrap_or_default();
    Some(format!(
        "{} {}",
        program.trim().trim_matches('"'),
        arguments.trim()
    ))
}

/// A whole `schtasks /Query /XML` document as the canonical record it describes, or `None` for a
/// document this reader cannot honestly claim to have read.
///
/// Pure, and separate from the query, so it can be asked about a **verbatim** document captured
/// from a real Task Scheduler rather than only about documents this file made up.
/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
/// crate links the library without `cfg(test)`, so a private item is invisible there.
pub fn provisioned_from_xml(doc: &str, expected: Option<&str>) -> Option<Provisioned> {
    let command = command_from_xml(doc)?;

    let when = when_from_xml(doc)?;
    let elevated = xml_element(doc, "Principals")
        .and_then(|p| xml_text(p, "RunLevel"))
        .is_some_and(|level| level == "HighestAvailable");
    // The task's own switch, which lives in `<Settings>`. A trigger carries an `<Enabled>` of its
    // own, so a document-wide search for the first one answers a different question. Absent
    // means armed — Task Scheduler omits the element on a task nobody has disabled, which is the
    // ordinary case and not a missing answer.
    let armed = xml_element(doc, "Settings")
        .and_then(|s| xml_text(s, "Enabled"))
        .map(|v| v == "true")
        .unwrap_or(true);
    let found = xml_element(doc, "Principals")
        .and_then(|p| xml_text(p, "UserId"))
        .filter(|user| !user.is_empty());
    let principal = expected.map(|want| match found {
        Some(found) if found == want => want.to_string(),
        other => other.unwrap_or_default().to_string(),
    });

    Some(Provisioned {
        spec: schtasks_spec(&command, &when, elevated, principal.as_deref()),
        armed,
    })
}

/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
/// crate links the library without `cfg(test)`, so a private item is invisible there.
pub fn schtasks_spec(command: &str, when: &str, elevated: bool, account: Option<&str>) -> String {
    let mut out = format!(
        "run: {}\nwhen: {}\nlevel: {}\n",
        command.trim(),
        when,
        if elevated {
            "HighestAvailable"
        } else {
            "LeastPrivilege"
        }
    );
    if let Some(user) = account {
        out.push_str(&format!("account: {user}\n"));
    }
    out
}

/// Task Scheduler's `/XML` output, whichever width it came back in.
///
/// The document declares `encoding="UTF-16"` and arrives here as bytes; on this machine a piped
/// read gives one byte per character, and a redirect to a file gives two. Deciding from the
/// bytes rather than from the header covers both without a table of which Windows does which.
/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
/// crate links the library without `cfg(test)`, so a private item is invisible there.
pub fn decode_xml(bytes: &[u8]) -> String {
    let utf16 =
        bytes.starts_with(&[0xFF, 0xFE]) || (bytes.len() >= 4 && bytes[1] == 0 && bytes[3] == 0);
    if !utf16 {
        return String::from_utf8_lossy(bytes).into_owned();
    }
    let body = bytes.strip_prefix(&[0xFF, 0xFE][..]).unwrap_or(bytes);
    // `as_chunks` rather than `chunks_exact(2)`: the pair arrives as `[u8; 2]`, so the two
    // indexes that could panic stop existing. A trailing odd byte is a truncated code unit and
    // is dropped either way.
    let (pairs, _odd_trailing_byte) = body.as_chunks::<2>();
    let units: Vec<u16> = pairs.iter().copied().map(u16::from_le_bytes).collect();
    String::from_utf16_lossy(&units)
}

/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
/// crate links the library without `cfg(test)`, so a private item is invisible there.
pub struct WindowsTaskProvisioner;

impl WindowsTaskProvisioner {
    fn task_name(name: &str) -> String {
        format!("Shall_{}", name)
    }

    fn principal(target: &TaskTarget) -> Option<String> {
        let account = target.account.as_ref()?;
        Some(account.sid().unwrap_or_else(|| account.name().to_string()))
    }

    /// **`pub` for the tests' sake** — the same exception `service_unit` documents: a test crate
    /// links the library without `cfg(test)`, so a private item is invisible there.
    pub fn command_line(config: &ScheduleConfig, shall_bin: &Path, target: &TaskTarget) -> String {
        format!(
            "{}{} {}",
            shall_bin.display(),
            target.roots.flag_spellings(),
            config.command
        )
    }

    /// The `/TR` a `schtasks /Create` hands over: the binary quoted (the line is one string
    /// Task Scheduler splits itself, and an unquoted path with a space is two programs), then
    /// the declaring run's roots flags, then the command.
    ///
    /// **The roots flags are written into the fire line, not only into the spec comparison.**
    /// `c7dd0b67` (#192) gave `command_line` the flags and missed this line, so a
    /// non-default-root Windows task re-registered on every sync and never recorded which
    /// config declared it — which the `#158` sweep needs to not delete another config's
    /// schedules (#182). The read-back (`command_from_xml`) trims the binary's quotes, so
    /// both sides of the standing comparison disagree only in the quotes that never reach it.
    /// **`pub` for the tests' sake**, like `command_line`.
    pub fn tr_command(config: &ScheduleConfig, shall_bin: &Path, target: &TaskTarget) -> String {
        format!(
            "\"{}\"{} {}",
            shall_bin.display(),
            target.roots.flag_spellings(),
            config.command
        )
    }

    /// Did `schtasks` refuse because this shell is not elevated?
    ///
    /// **One reading, because there were about to be two.** `/Create` and `/Delete` both answer
    /// `ERROR: Access is denied.` when the token is filtered, and that sentence names neither the
    /// cause nor the cure — it reads like a permissions problem with the *config*. `/Create` had
    /// been taught to say so; `/Delete` had not, and answered `the scheduled task … still exists
    /// after removal`, which is a description of the symptom by the one party who knows the cause
    /// (#173). Two copies of "contains `access is denied`" would have drifted the same way.
    ///
    /// The text, not the exit code: `schtasks` exits 1 for a task that does not exist as well, so
    /// the code cannot tell *refused* from *already gone*. Matched case-insensitively and on the
    /// phrase rather than the whole line, because the message is translated on a non-English
    /// Windows and the code page is not the one this file is written in.
    fn refused_for_elevation(e: &crate::core::Error) -> bool {
        e.to_string().to_lowercase().contains("access is denied")
    }

    /// The sentence a person needs when Task Scheduler says no: what to do, not what happened.
    fn needs_elevation(verb: &str, name: &str) -> crate::core::Error {
        crate::core::Error::Permission(format!(
            "{} the scheduled task `{}` needs an elevated shell — Windows Task Scheduler \
             refuses to {} one otherwise, whatever `/RU` says. Re-run `shall sync` from a \
             terminal opened with \"Run as administrator\".",
            verb, name, verb
        ))
    }
}

#[async_trait]
impl TaskProvisioner for WindowsTaskProvisioner {
    fn refuse_unsupported(&self, config: &ScheduleConfig) -> std::result::Result<(), String> {
        // Both live in the task's XML and `schtasks` has no flag for either, so accepting them
        // would mean accepting an option and writing a task without it.
        if config.jitter.is_some() {
            return Err(
                "`jitter` is a Task Scheduler XML setting (`RandomDelay`) and `schtasks` \
                        has no flag for it, so Shall cannot put it on the task it creates. \
                        Randomise inside the command the schedule invokes, or drop the option."
                    .into(),
            );
        }
        if config.persistent.is_some() {
            return Err(
                "`persistent` is a Task Scheduler XML setting (`StartWhenAvailable`) and \
                        `schtasks` has no flag for it, so Shall can neither set it nor promise \
                        what it defaults to. Drop the option on this machine."
                    .into(),
            );
        }
        Ok(())
    }

    fn rendered(
        &self,
        config: &ScheduleConfig,
        shall_bin: &Path,
        target: &TaskTarget,
    ) -> std::result::Result<Provisioned, String> {
        let args = map_cron_to_schtasks(&config.cron)?;
        let when = schtasks_when(&args).ok_or_else(|| {
            format!(
                "`{}` maps to a Task Scheduler trigger Shall cannot describe.",
                config.cron
            )
        })?;
        Ok(Provisioned {
            spec: schtasks_spec(
                &Self::command_line(config, shall_bin, target),
                &when,
                config.elevated == Some(true),
                Self::principal(target).as_deref(),
            ),
            armed: armed(config),
        })
    }

    async fn read_task(
        &self,
        executor: &CommandExecutor,
        name: &str,
        target: &TaskTarget,
    ) -> Reading {
        let tn = Self::task_name(name);
        let queried = executor
            .run("schtasks", &["/Query", "/TN", &tn, "/XML"], false)
            .await;
        let doc = match queried {
            Ok(out) => decode_xml(&out.stdout),
            Err(_) => {
                // Two reasons a query fails and only one of them is an answer. Asking
                // `schtasks` a question it can always answer separates them, which the error
                // text cannot: `ERROR: The system cannot find the file specified.` is
                // translated on a non-English Windows and matching it would make the reading
                // depend on the machine's display language.
                return match executor
                    .run("schtasks", &["/Query", "/FO", "CSV", "/NH"], false)
                    .await
                {
                    Ok(_) => Reading::Absent,
                    Err(_) => Reading::Unreadable,
                };
            }
        };

        match provisioned_from_xml(&doc, Self::principal(target).as_deref()) {
            Some(held) => Reading::Holds(held),
            None => Reading::Unreadable,
        }
    }

    /// `schtasks /Query` over the whole task store, filtered to the prefix Shall writes.
    ///
    /// There is no directory to scan on Windows: the store belongs to the scheduler, so the only
    /// listing is the query. `Shall_` is stripped to recover the declared name, and a query that
    /// fails is `None` — an unreadable store is not an empty one.
    async fn installed(
        &self,
        executor: &CommandExecutor,
        _target: &TaskTarget,
    ) -> Option<Vec<String>> {
        let out = executor
            .run("schtasks", &["/Query", "/FO", "CSV", "/NH"], false)
            .await
            .ok()?;
        let text = crate::utils::text::sanitize(&String::from_utf8_lossy(&out.stdout));
        let mut names: Vec<String> = Vec::new();
        for line in text.lines() {
            // `"\\Shall_nightly","N/A","Ready"` — the task name is the first field, quoted, and
            // may carry a folder prefix (`\` or `\SomeFolder\`).
            let Some(field) = line.split(',').next() else {
                continue;
            };
            let field = field.trim().trim_matches('"');
            let task = field.rsplit('\\').next().unwrap_or(field);
            let Some(name) = task.strip_prefix("Shall_") else {
                continue;
            };
            names.push(name.to_string());
        }
        names.sort();
        Some(names)
    }

    async fn installed_with_command(
        &self,
        executor: &CommandExecutor,
        _target: &TaskTarget,
    ) -> Option<Vec<HeldTask>> {
        let names = self.installed(executor, _target).await?;
        let mut tasks = Vec::with_capacity(names.len());
        for name in names {
            let tn = Self::task_name(&name);
            // One `/XML` query per task: the whole-store listing can be widened (`/V`), but its
            // CSV has no escaping story for a command with a comma, and `/XML` is the document
            // every other read-back already parses. A query that fails answers with an empty
            // argv and `certified: false` — a store with nothing left to say does not get to
            // call an unreadable task ours.
            let argv = match executor
                .run("schtasks", &["/Query", "/TN", &tn, "/XML"], false)
                .await
            {
                Ok(out) => {
                    let doc = decode_xml(&out.stdout);
                    command_from_xml(&doc)
                        .map(|command| crate::model::schedule::words(&command))
                        .unwrap_or_default()
                }
                Err(_) => Vec::new(),
            };
            tasks.push(HeldTask {
                name,
                argv,
                certified: false,
            });
        }
        Some(tasks)
    }

    async fn add_task(
        &self,
        executor: &CommandExecutor,
        config: &ScheduleConfig,
        shall_bin: &Path,
        target: &TaskTarget,
    ) -> Result<()> {
        let name = Self::task_name(&config.name);
        // Quoted for `/TR`, which takes one string and splits it itself; the read-back compares
        // the unquoted form Task Scheduler stores.
        //
        // **The roots flags ride in the same line** — systemd's `ExecStart` has carried them
        // since `c7dd0b67` (#192), and this line was the one renderer they never reached: the
        // *spec* comparison gained them, so a non-default-root Windows task re-registered on
        // every sync, and a Windows schedule never recorded which config declared it, which the
        // `#158` sweep needs to not delete another config's tasks (#182).
        let cmd = Self::tr_command(config, shall_bin, target);

        // Refused here, before anything is created: a schedule Task Scheduler cannot express
        // must not become the nearest one it can. `Refused` and not `Io` — Shall looked and
        // declined on purpose, which is exit code 3 (U21), and a script that retries on failure
        // must not retry this.
        let schedule = map_cron_to_schtasks(&config.cron)
            .map_err(|e| Error::Refused(format!("`schedule:{}`: {}", config.name, e)))?;

        if target.is_scoped() && !TaskTarget::can_register_for_another_account() {
            return Err(target.refuse_no_user_domain(
                &format!("schedule:{}", config.name),
                "Windows Task Scheduler",
                "registering a task for another account needs that account's password, and \
                 Shall does not hold it",
            ));
        }
        let principal = Self::principal(target);
        let mut args: Vec<&str> = vec!["/Create", "/TN", &name, "/TR", &cmd, "/F"];
        args.extend(schedule.iter().map(String::as_str));
        if config.elevated == Some(true) {
            args.extend(["/RL", "HIGHEST"]);
        }
        if let Some(user) = principal.as_deref() {
            args.extend(["/RU", user]);
        }

        executor.run("schtasks", &args, true).await.map_err(|e| {
            if Self::refused_for_elevation(&e) {
                Self::needs_elevation("creating", &name)
            } else {
                e
            }
        })?;

        // `/Create` always produces an armed task, so the declaration that says otherwise is
        // applied after it rather than instead of it.
        if !armed(config) {
            executor
                .run("schtasks", &["/Change", "/TN", &name, "/DISABLE"], true)
                .await?;
        }
        Ok(())
    }

    async fn remove_task(
        &self,
        executor: &CommandExecutor,
        name: &str,
        target: &TaskTarget,
    ) -> Result<()> {
        let tn = Self::task_name(name);
        // **The reason is kept, not discarded.** This was `let _ = …`, so the one party that
        // knew why the removal failed threw the answer away and then reported the symptom —
        // `Access is denied.` reached the user as "still exists after removal", which reads as a
        // scheduler problem and not as "this shell is not elevated" (#173). A task that is
        // already gone also exits non-zero, so the exit code cannot be the test; the text can,
        // and the end state decides what happens next either way.
        let tried = executor
            .run("schtasks", &["/Delete", "/TN", &tn, "/F"], true)
            .await;
        if self.is_task_active(executor, name, target).await {
            return Err(match &tried {
                Err(e) if Self::refused_for_elevation(e) => Self::needs_elevation("removing", name),
                Err(e) => Error::Io(format!(
                    "the scheduled task `{}` could not be removed and is still registered: {}\n\
                     Check `schtasks /Query /TN {}`.",
                    name, e, tn
                )),
                // It said it deleted the task and the task is still there: nothing answered for
                // this, and guessing at a cause would be worse than naming the two facts.
                Ok(_) => Error::Io(format!(
                    "`schtasks /Delete` reported success for `{}` and the task is still \
                     registered. Check `schtasks /Query /TN {}`.",
                    name, tn
                )),
            });
        }
        Ok(())
    }

    async fn is_task_active(
        &self,
        executor: &CommandExecutor,
        name: &str,
        _target: &TaskTarget,
    ) -> bool {
        let tn = Self::task_name(name);
        match executor
            .run("schtasks", &["/Query", "/TN", &tn], false)
            .await
        {
            Ok(o) => o.status.success(),
            Err(_) => false,
        }
    }
}
