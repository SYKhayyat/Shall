use crate::config::grammar::Options;
use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};

/// Whether a dated line is counting right now (SPEC II.7 rule 6).
///
/// Two keys, mirror images:
/// - `@expires` on a present line: present now, absent after.
/// - `@until` on an `absent:` line: absent now, present after.
///
/// Both are absolute (V.38). A duration cannot work in a file, because the machine reading
/// it next week has no idea when you wrote it — which is exactly why `@lease=2h` was inert.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dating {
    /// No date. Counts forever.
    Undated,
    /// Dated and still in force.
    Counting,
    /// Dated and its date has passed. **Stops counting** — it does not flip to the
    /// opposite meaning, it stops having an opinion.
    Lapsed,
}

impl Dating {
    pub fn counts(self) -> bool {
        !matches!(self, Dating::Lapsed)
    }

    pub fn is_dated(self) -> bool {
        !matches!(self, Dating::Undated)
    }
}

/// Classify a declaration's dating at `now`.
///
/// An unparseable date is `Undated` here rather than an error: the grammar already refused
/// it at parse time (`@expires=2h`), so anything reaching this is well-formed.
pub fn dating_of(options: &Options, now: DateTime<Utc>) -> Dating {
    for key in ["expires", "until"] {
        if let Some(raw) = options.one(key) {
            let Some(when) = parse_absolute(raw) else {
                continue;
            };
            return if now >= when {
                Dating::Lapsed
            } else {
                Dating::Counting
            };
        }
    }
    Dating::Undated
}

/// Parse the absolute forms II.2 accepts. Kept beside `dating_of` so the grammar's
/// validation and this reader cannot drift apart into accepting different sets.
pub fn parse_absolute(raw: &str) -> Option<DateTime<Utc>> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(raw) {
        return Some(dt.with_timezone(&Utc));
    }
    for fmt in ["%Y-%m-%dT%H:%M:%S", "%Y-%m-%dT%H:%M"] {
        if let Ok(naive) = NaiveDateTime::parse_from_str(raw, fmt) {
            return Some(Utc.from_utc_datetime(&naive));
        }
    }
    if let Ok(d) = NaiveDate::parse_from_str(raw, "%Y-%m-%d") {
        return Some(Utc.from_utc_datetime(&d.and_hms_opt(0, 0, 0)?));
    }
    None
}

/// Turn a duration someone typed into the absolute time a file can hold (V.38).
///
/// `--temp 2h` is a fine thing to type and an impossible thing to store: a file saying "2
/// hours" cannot be read, because the machine reading it does not know when you wrote it,
/// and it would mean something different every time it was read. The command line knows
/// `now`, so this is where the conversion belongs — and the line it writes says exactly
/// when, forever.
///
/// Accepts `s`, `m`, `h`, `d`, `w`.
pub fn absolute_after(now: DateTime<Utc>, duration: &str) -> Option<String> {
    let raw = duration.trim();
    let unit = raw.chars().last()?;
    let value: i64 = raw[..raw.len() - unit.len_utf8()].parse().ok()?;
    if value < 0 {
        return None;
    }
    let delta = match unit {
        's' => chrono::Duration::try_seconds(value),
        'm' => chrono::Duration::try_minutes(value),
        'h' => chrono::Duration::try_hours(value),
        'd' => chrono::Duration::try_days(value),
        'w' => chrono::Duration::try_weeks(value),
        _ => return None,
    }?;
    let at = now.checked_add_signed(delta)?;
    // The format `parse_absolute` reads back, to the minute: a lease is not a stopwatch,
    // and seconds in a file you read next month are noise.
    Some(at.format("%Y-%m-%dT%H:%M").to_string())
}
