//! Where a run's seconds went, per child command.
//!
//! [`latency`](crate::core::latency) measures the *total* and says when it crosses a budget.
//! That is enough to notice a 98-second `info` and not enough to act on one: the next question
//! is always which manager took the time, and until this existed the only way to answer it was
//! to time the managers by hand outside Shall and subtract.
//!
//! **Sum and wall clock are both reported, because their ratio is the parallelism.** Shall
//! spends its life waiting on other people's processes, so a run whose child time sums to 6 s
//! inside a 3.4 s wall clock is overlapping them 1.8×, and one whose ratio is 1.0 is asking
//! them one at a time. A breakdown that printed only the sum would hide the difference the
//! whole design turns on.
//!
//! Recording is off unless `--timings` asks for it: the lock is uncontended and the cost is a
//! `String` per child, but a measurement nobody requested is exactly the eager work this
//! module exists to find.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use once_cell::sync::Lazy;

/// When this process began, so every span can be placed on one timeline.
static START: Lazy<Instant> = Lazy::new(Instant::now);

static ENABLED: AtomicBool = AtomicBool::new(false);

static SPANS: Lazy<Mutex<Vec<Span>>> = Lazy::new(|| Mutex::new(Vec::new()));

/// One child command, and when it ran relative to the start of the process.
struct Span {
    label: String,
    at: Duration,
    took: Duration,
}

/// Start recording. Called once, before anything is dispatched.
pub fn enable() {
    Lazy::force(&START);
    ENABLED.store(true, Ordering::Relaxed);
}

pub fn is_enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

/// How long since recording began — the run's wall clock, near enough.
///
/// Measured from [`enable`] rather than from the dispatch of the verb, because config loading
/// and the model resolution happen before that and are exactly the part a user who asks "why
/// did it sit there doing nothing" is asking about.
pub fn elapsed() -> Duration {
    START.elapsed()
}

/// Mark the start of a child command. Returns `None` when recording is off, so a caller that
/// is not being measured allocates nothing.
pub fn begin() -> Option<Instant> {
    is_enabled().then(Instant::now)
}

/// Record a finished child command.
///
/// `label` is the program and its first argument — `winget list`, not the whole argv. The
/// package names on an `install` line are the part that differs between two runs of the same
/// command, and a breakdown keyed by them would have one row per package instead of one row
/// per thing the run actually waited on.
pub fn end(started: Option<Instant>, cmd: &str, args: &[String]) {
    let Some(started) = started else { return };
    record(
        cmd,
        args,
        started.saturating_duration_since(*START),
        started.elapsed(),
    );
}

/// The label a program and its argv are gathered under.
fn label_of(cmd: &str, args: &[String]) -> String {
    match args.first() {
        Some(first) => format!("{} {}", cmd, first),
        None => cmd.to_string(),
    }
}

/// Record a span whose start and duration are given rather than measured.
///
/// **The seam that makes this module testable without a clock.** Its aggregation — the label
/// keying, the slowest-first order, the wave count — is arithmetic, and a test that produced
/// its inputs with `thread::sleep` was asserting on the scheduler instead: two 5 ms sleeps sum
/// past one 30 ms sleep whenever Windows' 15.6 ms timer granularity and a loaded machine
/// conspire, which is precisely when the suite runs (AU5). Timing is now measured in exactly
/// one place — [`end`] — and tested with one loose assertion that cannot order-invert.
fn record(cmd: &str, args: &[String], at: Duration, took: Duration) {
    // Poisoned is recovered, not dropped — the same policy `summary` applies. A panic between
    // two spans must not erase every span before it from the one report that explains the run.
    let mut spans = match SPANS.lock() {
        Ok(s) => s,
        Err(poisoned) => poisoned.into_inner(),
    };
    spans.push(Span {
        label: label_of(cmd, args),
        at,
        took,
    });
}

/// What a run spent, gathered per label.
pub struct Row {
    pub label: String,
    pub calls: usize,
    pub total: Duration,
    pub longest: Duration,
    /// When the first call started, measured from [`enable`].
    ///
    /// Aggregating durations answers *what* was slow and cannot answer *whether it waited*.
    /// Two runs with identical rows — one overlapping every child, one running them in file
    /// order — are the same table until the start offsets are printed beside it.
    pub first_at: Duration,
    /// When the last call finished.
    pub last_end: Duration,
}

/// The rows, slowest first, plus the wall clock and the summed child time.
///
/// Returned rather than printed so a test can assert on it without capturing stderr.
pub fn summary() -> (Vec<Row>, Duration, Duration) {
    let spans = match SPANS.lock() {
        Ok(s) => s,
        Err(poisoned) => poisoned.into_inner(),
    };

    let mut rows: Vec<Row> = Vec::new();
    let mut summed = Duration::ZERO;
    for span in spans.iter() {
        summed += span.took;
        match rows.iter_mut().find(|r| r.label == span.label) {
            Some(row) => {
                row.calls += 1;
                row.total += span.took;
                row.longest = row.longest.max(span.took);
                row.first_at = row.first_at.min(span.at);
                row.last_end = row.last_end.max(span.at + span.took);
            }
            None => rows.push(Row {
                label: span.label.clone(),
                calls: 1,
                total: span.took,
                longest: span.took,
                first_at: span.at,
                last_end: span.at + span.took,
            }),
        }
    }
    rows.sort_by(|a, b| b.total.cmp(&a.total).then_with(|| a.label.cmp(&b.label)));

    // The last child to finish, not the process's own elapsed time: this is called from
    // `main` before the exit path, and the caller's own wall clock is the honest total.
    let span_end = spans
        .iter()
        .map(|s| s.at + s.took)
        .max()
        .unwrap_or_default();
    (rows, span_end, summed)
}

/// How many times the run went completely quiet — no child running at all — and started again.
///
/// One wave means everything overlapped. `n` waves means the run stopped `n - 1` times to wait
/// for an answer before it could ask the next question, and that is the difference between a
/// low overlap ratio caused by *slow* children and one caused by *sequenced* ones. Counted here
/// rather than eyeballed off the start offsets, because that is the reading everyone gets wrong.
pub fn waves() -> usize {
    let spans = match SPANS.lock() {
        Ok(s) => s,
        Err(poisoned) => poisoned.into_inner(),
    };
    let mut intervals: Vec<(Duration, Duration)> =
        spans.iter().map(|s| (s.at, s.at + s.took)).collect();
    intervals.sort_by_key(|(at, _)| *at);

    let mut waves = 0;
    let mut open_until = Duration::ZERO;
    for (at, end) in intervals {
        // `>=`, not `>`: a child that starts exactly as the last one ends did wait for it.
        if at >= open_until {
            waves += 1;
            open_until = end;
        } else {
            open_until = open_until.max(end);
        }
    }
    waves
}

/// Print the breakdown to stderr.
///
/// stderr, not stdout: `shall eval --timings | jq` must still get JSON, and a measurement
/// written into the answer is a measurement that breaks every caller parsing it.
pub fn report(wall: Duration) {
    if !is_enabled() {
        return;
    }
    let (rows, _, summed) = summary();

    if rows.is_empty() {
        eprint!("{}", render(&[], wall, summed, 0));
        return;
    }
    let waves = waves();
    eprint!("{}", render(&rows, wall, summed, waves));
}

/// How many distinct programs a set of rows came from.
///
/// The program is the first token of the label, because [`label_of`] builds the label as
/// *program + first argument*. `npm list` and `npm prefix` are two child commands from one
/// program; `conda list` is a child command from another.
///
/// **This is a count of processes, not of backends.** Two limits, stated rather than papered over:
/// one backend that shells out to two different programs counts as two, and two backends that
/// happen to share a program count as one. Neither has been observed, and neither can be settled
/// without backend attribution in [`Span`] — which is the next step for `#107`, not something this
/// function pretends to know.
pub fn distinct_programs(rows: &[Row]) -> usize {
    rows.iter()
        .map(|r| r.label.split_whitespace().next().unwrap_or(&r.label))
        .collect::<std::collections::BTreeSet<_>>()
        .len()
}

/// The report as a string, so the per-child table can be asserted on.
///
/// **Separated from [`report`] because the table is evidence and evidence nobody can read is not
/// evidence.** `report` prints, and a printed table can only be checked by capturing a process's
/// stderr — which is why `#107` sat unanswerable for as long as it did: the names of the children
/// were being printed on a CI run, by a gate, and thrown away by the panic message that quoted
/// only the summary line. Rendering separately makes "the report names every child" a unit test
/// rather than a thing somebody has to notice in a log.
///
/// Empty `rows` renders the no-children line, so a caller cannot report an empty run as though it
/// had asked nobody anything while `rows` was merely unread.
pub fn render(rows: &[Row], wall: Duration, summed: Duration, waves: usize) -> String {
    let mut out = String::new();

    if rows.is_empty() {
        out.push_str(&format!(
            "\nTimings: {:.2}s wall, no child commands — this run asked no package manager \
             anything.",
            wall.as_secs_f64()
        ));
        return out;
    }

    let calls: usize = rows.iter().map(|r| r.calls).sum();
    // Below 1.0 only when the children were genuinely serial and the run spent time outside
    // them; it is a ratio of sums, so it never divides by zero here (rows is non-empty).
    let overlap = summed.as_secs_f64() / wall.as_secs_f64().max(f64::EPSILON);

    out.push_str(&format!(
        "\nTimings: {:.2}s wall · {} child command(s) summing to {:.2}s · {:.1}x overlap · {} \
         wave(s)",
        wall.as_secs_f64(),
        calls,
        summed.as_secs_f64(),
        overlap,
        waves,
    ));
    // **Child commands and programs are counted separately, because conflating them is what made
    // `#107` unanswerable.** `shall export` on a five-manager host spawns seven child commands —
    // `npm list` and `npm prefix`, `pipx list` and `pipx environment` — and the gate read that as
    // "7 managers", then handed 7 to `Shape::wave_ceiling`, which is calibrated on managers. A
    // number that is neither a count of managers nor a count of commands is a number nobody can
    // reason about.
    //
    // The distinct count is of **programs**, not managers, and deliberately so: a manager is a
    // Shall-side concept, while a span records a process, and the instrument does not know which
    // backend asked. Naming it `managers` would be the same overclaim one level down. Where the
    // two counts differ, that difference is the finding, so it is printed rather than left for a
    // reader to reconstruct.
    let programs = distinct_programs(rows);
    if programs < calls {
        out.push_str(&format!(
            "\n  {} child command(s) came from {} distinct program(s) — some program is asked \
             more than once per run, so the child count is not a manager count",
            calls, programs
        ));
    }
    if waves > 1 {
        out.push_str(&format!(
            "\n  {} wave(s) means the run went quiet {} time(s) — nothing was running, because \
             something had to be answered before the next question could be asked.",
            waves,
            waves - 1,
        ));
    }
    out.push_str("\n  (only commands Shall spawns are counted; its own parsing is the remainder)");
    out.push_str(&format!("\n  {:>7}  {:>7}   command", "at", "took"));
    for row in rows {
        let calls = if row.calls == 1 {
            String::new()
        } else {
            format!(
                "  ({}x, longest {:.2}s, last ended {:.2}s)",
                row.calls,
                row.longest.as_secs_f64(),
                row.last_end.as_secs_f64(),
            )
        };
        out.push_str(&format!(
            "\n  {:>6.2}s  {:>6.2}s   {}{}",
            row.first_at.as_secs_f64(),
            row.total.as_secs_f64(),
            row.label,
            calls,
        ));
    }
    out.push('\n');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The recorder is global, so these run as one test rather than racing each other over it.
    #[test]
    fn a_run_is_gathered_per_label_slowest_first_and_off_by_default() {
        // Off by default: `begin` hands back nothing, and `end` records nothing from it.
        assert!(!is_enabled());
        assert!(begin().is_none());
        end(None, "winget", &["list".to_string()]);
        assert!(
            summary().0.is_empty(),
            "a disabled recorder recorded a span"
        );

        enable();
        assert!(is_enabled());

        // Given, not slept for. The durations below are the test's subject matter; producing
        // them with a sleep made the assertions a measurement of the host's scheduler.
        let ms = Duration::from_millis;
        record("winget", &["list".to_string()], ms(0), ms(30));
        record(
            "cargo",
            &["install".to_string(), "--list".to_string()],
            ms(30),
            ms(5),
        );
        record(
            "cargo",
            &["install".to_string(), "--list".to_string()],
            ms(35),
            ms(5),
        );
        // No args at all still names the program, rather than producing an empty row.
        record("emacs", &[], ms(40), ms(1));

        let (rows, _, summed) = summary();
        assert_eq!(rows.len(), 3, "three distinct labels, not one row per call");

        // Slowest first.
        assert_eq!(rows[0].label, "winget list");
        assert_eq!(rows[0].calls, 1);

        // Keyed by program + FIRST arg, so `--list` does not open a second row, and the two
        // calls gather into one.
        let cargo = rows
            .iter()
            .find(|r| r.label == "cargo install")
            .expect("cargo install row");
        assert_eq!(cargo.calls, 2);
        assert!(
            cargo.total >= cargo.longest,
            "total must include the longest"
        );

        assert!(rows.iter().any(|r| r.label == "emacs"));

        assert!(summed >= rows[0].total, "the sum covers every row");

        // Every span above ran to completion before the next began, so each is its own wave.
        // This is the reading that matters: same rows, same totals, and a serial run is only
        // distinguishable from an overlapped one here.
        assert_eq!(waves(), 4, "four children, none overlapping, is four waves");

        let winget = rows.iter().find(|r| r.label == "winget list").unwrap();
        assert!(
            winget.last_end >= winget.first_at + winget.longest,
            "a row's span must cover its own longest call"
        );

        end_records_the_time_that_actually_passed();
    }

    /// The half [`record`] cannot cover: that [`end`] measures a real elapsed interval.
    ///
    /// One span, one loose lower bound, nothing compared against another span — so there is no
    /// ordering for a busy scheduler to invert. A 30 ms sleep that records less than 10 ms is
    /// not a slow machine, it is a broken clock.
    ///
    /// **Last, and inside the same test.** The recorder is process-global; a second `#[test]`
    /// would run on another thread and add a row to the table the assertions above are
    /// counting.
    /// **The rendered report names every child, which is the whole of what `#107` was missing.**
    ///
    /// The per-child table has always been printed; `report` threw it into stderr where the
    /// fan-out gate captured the process output and then quoted only the summary line, so a
    /// failing run named a count and an overlap ratio and not one manager. That is why the issue
    /// could say "which seven commands these are is the entire question" and never answer it: the
    /// answer was in the string it had in hand and was discarded.
    ///
    /// Asserted on every label, and on the start offsets, because the offsets are what turn "seven
    /// children" into "these seven, asked in this order, at these times" — the difference between a
    /// count and a diagnosis.
    #[test]
    fn the_rendered_report_names_every_child_and_when_it_started() {
        let ms = Duration::from_millis;
        let rows = vec![
            Row {
                label: "solus".to_string(),
                calls: 1,
                total: ms(300),
                longest: ms(300),
                first_at: ms(0),
                last_end: ms(300),
            },
            Row {
                label: "winget".to_string(),
                calls: 2,
                total: ms(900),
                longest: ms(500),
                first_at: ms(10),
                last_end: ms(910),
            },
        ];
        let text = render(&rows, ms(920), ms(1200), 7);

        // The summary line stays exactly as it was: the gate parses `child command(s)`,
        // `x overlap` and ` wave(s)` out of it, and a report that answers the diagnosis by
        // breaking its own parser would trade one unreadable failure for another.
        assert!(
            text.contains("3 child command(s) summing to 1.20s"),
            "the aggregate line is what the gate parses; it changed:\n{text}"
        );
        for label in ["solus", "winget"] {
            assert!(
                text.contains(label),
                "`{label}` was asked and the report does not say so, which is the `#107` \
                 failure this test exists to prevent:\n{text}"
            );
        }
        // The start offsets, not just the names: three children in one wave and seven waves is
        // the serialisation itself.
        assert!(
            text.contains("0.00s") && text.contains("0.01s"),
            "the table reports when each child started, which is how a serial run is told from an \
             overlapped one:\n{text}"
        );
        // A label called more than once says so, or a repeated question reads as one.
        assert!(
            text.contains("(2x, longest 0.50s"),
            "a label asked twice reports both calls, since `sbom` asking two questions of one \
             manager is not the same as asking one:\n{text}"
        );

        // **Seven child commands, five programs — the exact shape `#107` misread as seven
        // managers.** The two `npm` rows and the two `pipx` rows are one program each, so the
        // report has to say so: `Shape::wave_ceiling` is calibrated on managers, and the gate was
        // handing it a child count and calling it a manager count.
        let programs = distinct_programs(&rows);
        assert_eq!(calls_of(&rows), 3, "three child commands in this fixture");
        assert_eq!(
            programs, 2,
            "`solus` and `winget` are two programs however often each is asked; a program asked \
             twice is still one program"
        );

        // And the empty run still says it asked nobody, rather than rendering an empty table that
        // reads like a truncated one.
        let empty = render(&[], ms(50), Duration::ZERO, 0);
        assert!(
            empty.contains("no child commands"),
            "an empty run must say so in words:\n{empty}"
        );
        assert!(
            !empty.contains("command\n"),
            "an empty run must not print a table header, which would look like rows lost:\n{empty}"
        );
    }

    /// The child-command count, which is the sum of `calls` and is what `render` prints.
    fn calls_of(rows: &[Row]) -> usize {
        rows.iter().map(|r| r.calls).sum()
    }

    fn end_records_the_time_that_actually_passed() {
        let started = begin();
        assert!(started.is_some(), "recording is on");
        std::thread::sleep(Duration::from_millis(30));
        end(started, "shall-timing-probe", &["sleep".to_string()]);

        let (rows, _, _) = summary();
        let row = rows
            .iter()
            .find(|r| r.label == "shall-timing-probe sleep")
            .expect("the span was recorded");
        assert!(
            row.total >= Duration::from_millis(10),
            "a 30ms child was recorded as {:?}",
            row.total
        );
    }
}
