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
//! **The ratio is not the only reading, because it is a measurement of the host as much as of
//! the scheduler.** `sum / wall` falls when children are slow to *start*, and a machine under a
//! container runtime, an antivirus or a cold page cache can spend as long creating a process as
//! the process then runs. [`width`] is the reading that cannot be fooled: it counts how many
//! children were in flight at the busiest instant, so a `for` loop that awaits each child is 1
//! however long each child took. Both are printed on the same line.
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
/// **`pub` for the integration tests' sake.** The timing mirror supplies deterministic spans
/// without sleeping to manufacture scheduler-dependent durations.
#[doc(hidden)]
pub fn record(cmd: &str, args: &[String], at: Duration, took: Duration) {
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
    schedule().0
}

/// The widest the run ever got: how many child commands were in flight at the same instant.
///
/// **The one reading that survives a host that spawns processes slowly.** The overlap ratio is
/// `sum(child time) / wall`, so it reads the host's process-startup cost as well as Shall's
/// scheduling: on a runner where `CreateProcess` takes about as long as the child itself runs,
/// every child's span begins late, the ratio collapses towards 1.0, and a perfectly good fan-out
/// is indistinguishable from a `for` loop that awaits each child. A width cannot be fooled that
/// way, because it is computed from the spans the run actually produced: a loop asks one child
/// at a time and its width is exactly 1, whatever a second of it costs on the host.
///
/// Measured on the Windows CI runner that made this function necessary: `shall sbom` asked seven
/// child commands, summed them to 5.23s, finished in 4.25s — 1.2x overlap, under the 1.25x floor
/// — while two children were demonstrably in flight together. `shall list` on the same host and
/// the same managers passed the floor, which is what makes the ratio, and not the fan-out, the
/// thing that could not see: `sbom` declares five managers, `list` probes seventeen, and the
/// ratio is bounded by the fan-out's width rather than by whether the fan-out exists.
pub fn width() -> usize {
    schedule().1
}

/// Both readings from one pass over the spans, because they answer one question.
///
/// `waves` is how many separate times the run had work running, `width` is how many children
/// shared the busiest of those times. One implementation and not two: a serial run is `width 1`
/// *and* one wave per child, and two sweeps that disagreed about the tie rule would be two
/// different answers to "did these overlap".
pub fn schedule() -> (usize, usize) {
    let spans = match SPANS.lock() {
        Ok(s) => s,
        Err(poisoned) => poisoned.into_inner(),
    };
    let mut intervals: Vec<(Duration, Duration)> =
        spans.iter().map(|s| (s.at, s.at + s.took)).collect();
    intervals.sort_by_key(|(at, end)| (*at, *end));

    let mut waves = 0;
    let mut open_until = Duration::ZERO;
    for (at, end) in &intervals {
        // `>=`, not `>`: a child that starts exactly as the last one ends did wait for it.
        if *at >= open_until {
            waves += 1;
            open_until = *end;
        } else {
            open_until = open_until.max(*end);
        }
    }

    // Ends first at a tie, so this uses exactly the convention the wave count above uses: a
    // child that begins at the instant another finishes waited for it and is not in flight
    // alongside it. Without the `<=` the width would report 2 for every hand-off in a serial
    // loop, which is the reading this number exists to distinguish.
    let mut in_flight: std::collections::BinaryHeap<std::cmp::Reverse<Duration>> =
        std::collections::BinaryHeap::new();
    let mut width = 0;
    for (at, end) in &intervals {
        while in_flight
            .peek()
            .is_some_and(|std::cmp::Reverse(finishes)| finishes <= at)
        {
            in_flight.pop();
        }
        in_flight.push(std::cmp::Reverse(*end));
        width = width.max(in_flight.len());
    }

    (waves, width)
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
        eprint!("{}", render(&[], wall, summed, 0, 0));
        return;
    }
    let (waves, width) = schedule();
    eprint!("{}", render(&rows, wall, summed, waves, width));
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
pub fn render(
    rows: &[Row],
    wall: Duration,
    summed: Duration,
    waves: usize,
    width: usize,
) -> String {
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

    // **`wide` is appended, never substituted.** Every existing reader finds `child command(s)`,
    // `x overlap` and ` wave(s)` where it found them before; the one number the ratio cannot
    // report — how many children actually shared the busiest instant — is added to the end of
    // the same sentence, so a report and a gate cannot read two different lines.
    out.push_str(&format!(
        "\nTimings: {:.2}s wall · {} child command(s) summing to {:.2}s · {:.1}x overlap · {} \
         wave(s) · {} wide",
        wall.as_secs_f64(),
        calls,
        summed.as_secs_f64(),
        overlap,
        waves,
        width,
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
