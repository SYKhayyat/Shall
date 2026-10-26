//! How long a command may take before Shall says so.
//!
//! Nothing measured latency, which is how a 98-second `shall info cargo:ripgrep` shipped —
//! answering `not found in any available backend` about a package `shall search ripgrep` found
//! in the same tree, seconds later (E14/E15). W14 fixed that one `info`; the budget it asked
//! for was never built, so nothing would notice the next one.
//!
//! **The split is diagnostic, not arbitrary.** Measured on Windows with 24 ready backends,
//! debug build, a fixture with 111 adopted packages:
//!
//! ```text
//! policy / vars / eval / check config        0.13 – 0.32 s
//! list                                       3.4  – 3.9  s
//! check health                               4.3  – 5.4  s
//! check                                      8.5  – 18.3 s
//! ```
//!
//! A command that only reads files is fast on every machine. A command that asks every manager
//! costs whatever the managers cost, and that is a fact about the host rather than about Shall.
//! So the budget is per **class**, and only the two classes whose cost Shall controls carry a
//! hard one.
//!
//! **The numbers are ceilings a collapse crosses, not targets.** A budget tight enough to
//! police 3 seconds against 5 on a shared CI runner is a gate that goes red on load, and a gate
//! that goes red on load is a gate people learn to ignore — `lifecycle-floor.txt` says the same
//! thing about guessed constants. These are set an order of magnitude above what was measured,
//! so what crosses them is the 98-second shape and not a busy afternoon.

use std::time::Duration;

/// What a command has to do before it can answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    /// Reads the config and answers. Asks no manager anything, so its cost is Shall's alone.
    ConfigOnly,
    /// Asks exactly one manager, because the user named it — `info cargo:ripgrep`. The
    /// qualifier is the whole point: it is what makes this cheaper than asking everybody, and
    /// E14 was that the qualifier did not narrow the probe.
    OneBackend,
    /// Asks every ready manager. Its cost belongs to the host — 24 managers on this box — so it
    /// is measured and reported, never failed.
    EveryBackend,
    /// Changes the machine. Bounded by what a package manager does, which is unbounded.
    Mutating,
}

/// What a fan-out has to look like, for the classes whose seconds belong to the host.
///
/// **The budget the wall clock cannot express.** `EveryBackend` costs whatever 24 managers cost,
/// so a ceiling in seconds is either useless or red on a busy afternoon — which is why it was
/// `None`. But Shall's *share* of that is measurable and it is already measured: `--timings`
/// computes the overlap ratio and the wave count on every run that asks for them, and nothing
/// read either. So the regression this could not see is the important one: a change that
/// serialises a fan-out drops overlap from 6.3× to 1.2×, the wall clock stays inside a budget of
/// `None`, and it stays there for ever.
///
/// **Collapse detectors, not targets, and the difference cost a red gate to learn.**
///
/// The first version of this carried `min_overlap: 2.0` and `max_waves: 2`, taken from the one
/// host it was written on — Windows, 24 ready backends, 6.3× and 2 waves. ubuntu-latest runs 16
/// child commands and reported **2.0× and 3 waves**: sitting exactly on one floor and over the
/// other, on a machine doing nothing wrong. That is the mistake `lifecycle-floor.txt` is entirely
/// about — *"the honest number varies by host, and a number guessed once is the kind of constant
/// this repo keeps discovering was wrong"* — committed while building the gate.
///
/// So neither number is absolute now. What is actually being asserted is that the fan-out is not
/// **serial**, and a serial run has an exact signature: overlap ≈ 1.0 and one wave per child.
/// Both bounds are expressed against that, which makes them true on a host nobody has measured.
///
/// **`min_overlap` came down from 1.5 to 1.25 on 2026-08-18, and the reading that moved it is the
/// kind this doc block already predicts.** The `cargo mutants` job runs the whole suite as its
/// unmutated baseline under `-j2`, so it is the only place this gate is measured on a host
/// running two test suites at once. There, `sbom` reported **1.4× over 21 children and 5 waves**
/// — under the floor, over nothing else, and green on the same machine's control run of `list`.
/// A shard failed its baseline for it while two other shards of the same tree passed.
///
/// 1.5 was set from healthy hosts, where the lowest legitimate reading is 2.0. That is a target
/// wearing a detector's clothes: a contended host is not a serialised fan-out, and 1.4× is 40%
/// above the serial signature this rule says it is testing for. 1.25 keeps a quarter's margin
/// over collapse and stops reporting a busy machine as a design regression. The wave ceiling is
/// untouched and is the load-independent half of the pair — it passed on the run that failed.
#[derive(Debug, Clone, Copy)]
pub struct Shape {
    /// Summed child time over wall clock. A collapse to serial is ~1.0; the lowest legitimate
    /// reading seen on any host is 2.0, so this sits between them.
    ///
    /// **A floor only, and a floor on this number can be satisfied by making things worse.**
    /// The ratio is `sum(child time) / wall`, and contention inflates the numerator: measured
    /// over the same 23 children, width 20 spent 676.6s of child time against width 4's
    /// 182.5s — 3.7× the total work — and the ratio *rose* from 1.6× to 8.3× for it. Wall clock
    /// improved monotonically, so the parallelism earns its keep and this is not a budget on
    /// the design. It does mean the floor cannot be the only thing read: the gate pairs it with
    /// an arithmetic ceiling (a run cannot average more children in flight than it has) and
    /// prints the numerator, so the one regression neither catches — every child getting slower
    /// at constant concurrency, which moves `sum` and `wall` together — is at least visible in
    /// a diff of two runs.
    pub min_overlap: f64,
    /// The wave ceiling as a fraction of the child count: waves may not exceed
    /// `children / waves_per_child`. A serial run has one wave per child, so the ceiling scales
    /// with the fan-out instead of pinning a number measured on somebody's desk.
    pub waves_per_child: usize,
    /// The floor under that fraction, so a small fan-out is not policed into failure: four
    /// children over a divisor of three would allow one wave, which no real run achieves.
    pub min_waves_allowed: usize,
    /// Below this many child commands the ratio is not a measurement of anything — three
    /// managers on a bare CI runner cannot overlap 2×, and a gate that says they should is a
    /// gate people learn to ignore.
    pub min_children: usize,

    /// How far the *achievable* overlap has to clear the floor before the floor is read against
    /// it, as a multiple.
    ///
    /// **The ratio's ceiling is `summed / slowest`, and a host whose children are skewed has a
    /// low one.** Overlap is `sum(child time) / wall`, and the smallest wall any scheduler can
    /// reach is the slowest child — every other child hides inside it. So `summed / slowest` is
    /// the most any host can score, and when one child holds most of the summed time that
    /// approaches 1.0 and the floor becomes unreachable *by a perfect scheduler*. Measured on a
    /// five-manager host where `emacs --batch` alone held 0.36s of 0.46s summed: a ceiling of
    /// 1.28× against a floor of 1.25×, with every run starting all five children at the same
    /// instant and finishing in one wave (`PLAN.md` #90).
    ///
    /// **So the question is not "is the floor right" but "does this host have a measurement",
    /// and the answer is no.** Same disease as the 1.5→1.25 move above, with the other
    /// denominator: that one was contention inflating `sum`, this one is skew pinning `wall`.
    /// Both make a correct scheduler look serial.
    ///
    /// **A quarter's headroom, and not a lower floor.** Lowering the floor to the ceiling would
    /// be the same mistake twice: on a skewed host a genuinely *serial* run scores ~1.0 and would
    /// pass a floor set near 1.28, which trades a false failure for a false pass. Skipping says
    /// the measurement does not exist, and the load-independent half of this pair still runs —
    /// `waves` is 1 for a perfect fan-out and one-per-child for a serial one whatever the child
    /// durations are, so collapse stays detectable on exactly the host where the ratio gave up.
    pub overlap_headroom: f64,
}

impl Shape {
    /// How many waves this many children may take before the run is serial enough to fail.
    pub fn wave_ceiling(&self, children: usize) -> usize {
        (children / self.waves_per_child.max(1)).max(self.min_waves_allowed)
    }

    /// Whether this many child commands is enough for the overlap ratio to be a measurement.
    ///
    /// **One condition, two callers, and that is the point.** [`Shape::min_children`] is read
    /// here and nowhere else, so [`shape_violation`] and the reporting arm cannot come to
    /// disagree about when the rule is in force. When they could, the failure mode is the worst
    /// kind this repository keeps finding: the rule says "not measurable", the reporter says "no
    /// violation", and the run is green on a question nobody asked.
    ///
    /// **The child count is only the first of two ways this host has no measurement**, and it is
    /// the one that can be read without a duration — see [`shape_measurement_gap`], which is the
    /// whole answer and delegates here.
    pub fn is_measurable(&self, children: usize) -> bool {
        children >= self.min_children
    }
}

/// The most overlap ratio any scheduler can reach over these children: `summed / slowest`.
///
/// Every child but the slowest hides inside the slowest one's wall time, so a fully concurrent
/// run's wall clock *is* the slowest child. That is the ceiling the floor has to be read against,
/// and it is a property of the children's durations rather than of the scheduler — which is what
/// makes it the difference between "this fan-out collapsed" and "this host cannot express the
/// difference".
pub fn overlap_ceiling(summed: Duration, slowest: Duration) -> f64 {
    summed.as_secs_f64() / slowest.as_secs_f64().max(f64::EPSILON)
}

/// Why the overlap ratio cannot be read against the floor on this host, or `None` when it can.
///
/// **One predicate, two callers, and both of them the reason this is a function.** The rule and
/// the reporter must not be able to disagree about when a bound is in force — when they can, the
/// failure mode is the one this repo keeps finding: the rule says "not measurable", the reporter
/// says "no violation", and the run is green on a question nobody asked. That is why the skip
/// comes out of here as a sentence rather than as a `bool` some caller is free to invent a
/// different reason for.
///
/// Two ways to have no measurement, and the second is the one that was missing:
///
/// 1. **Too few children.** The original condition, and [`Shape::is_measurable`].
/// 2. **Children too skewed for the floor to be reachable, in a run that demonstrably did
///    overlap.** The ratio's ceiling is `summed / slowest` ([`overlap_ceiling`]); when that does
///    not clear the floor by [`Shape::overlap_headroom`], no scheduler on this host could have
///    scored what the floor asks for. `PLAN.md` #90.
///
/// **The `waves` condition is not a detail — it is what stops the exemption from being the blind
/// spot it looks like.** A run that collapsed to serial has `slowest ≈ summed` *by construction*,
/// so its ceiling is ~1.0 and it would claim the skew exemption and never be judged: the gate
/// would exempt precisely the run it exists to catch. One wave is the evidence that the children
/// really did start together, and a run that finished in one wave cannot have serialised itself.
/// So the exemption is granted on evidence of concurrency and withheld from everything else.
///
/// **A gap here costs the wave check nothing.** `waves` does not depend on child durations at
/// all — one wave is a perfect fan-out and one wave per child is a serial loop — so the
/// load-independent half of the pair stays in force on exactly the host where the ratio gave up.
pub fn shape_measurement_gap(
    shape: Shape,
    children: usize,
    summed: Duration,
    slowest: Duration,
    waves: usize,
) -> Option<String> {
    if !shape.is_measurable(children) {
        return Some(format!(
            "{children} child command(s), under the {} the overlap ratio needs before it measures \
             anything rather than reading as serial",
            shape.min_children
        ));
    }
    // Without this the gate would exempt every serial run, because a serial run's slowest child
    // is the whole run.
    if waves > 1 {
        return None;
    }
    let ceiling = overlap_ceiling(summed, slowest);
    let reachable = shape.min_overlap * shape.overlap_headroom;
    (ceiling < reachable).then(|| {
        format!(
            "the slowest child took {:.2}s of the {:.2}s these {} child command(s) summed to, so \
             no scheduler here can score above {ceiling:.2}x — under the {reachable:.2}x this \
             host would need before a {:.2}x floor measures anything rather than reading as \
             serial. Every child finished in one wave, so the run did overlap; the wave count \
             does not depend on how long a child took and is still checked.",
            slowest.as_secs_f64(),
            summed.as_secs_f64(),
            children,
            shape.min_overlap
        )
    })
}

impl Class {
    /// The ceiling, or `None` where the cost is the host's rather than Shall's.
    pub fn budget(self) -> Option<Duration> {
        match self {
            Class::ConfigOnly => Some(Duration::from_secs(5)),
            Class::OneBackend => Some(Duration::from_secs(15)),
            Class::EveryBackend | Class::Mutating => None,
        }
    }

    /// The shape budget, for the class whose cost is the host's but whose *scheduling* is not.
    ///
    /// `Mutating` has no [`Shape`] and is not exempt — it is measured by
    /// [`scheduling_violation`] instead, against a bound this heuristic cannot express.
    /// `waves_per_child` is a guess about the shape of the work; a mutating run's shape is not a
    /// guess, it is the plan's own **critical-path depth**, which the engine holds and this
    /// layer never sees. So the engine reports its own shape rather than being inspected from
    /// here, and the rule is exact: a scheduler may not go quiet more often than the dependency
    /// graph forces it to.
    ///
    /// Every number here is a collapse detector — see [`Shape`] for what the first draft's
    /// targets cost. Four readings, from three platforms, all of them healthy:
    ///
    /// ```text
    /// Windows, release   23 children   6.3x   2 waves
    /// Windows, debug     23 children   5.7x   2 waves
    /// ubuntu-latest      16 children   2.0x   3 waves
    /// macos-latest        9 children   1.9x   3 waves
    /// ```
    ///
    /// A serial run is 1.0× with one wave per child. The floor sits at 1.5× — below every
    /// reading above and half again above serial — and the wave ceiling at half the child count
    /// with a floor of four, which still catches a nine-child run that took nine waves while
    /// leaving the honest three alone.
    pub fn shape(self) -> Option<Shape> {
        match self {
            Class::EveryBackend => Some(Shape {
                min_overlap: 1.25,
                waves_per_child: 2,
                min_waves_allowed: 4,
                min_children: 4,
                overlap_headroom: 1.25,
            }),
            Class::ConfigOnly | Class::OneBackend | Class::Mutating => None,
        }
    }

    /// Which class a subcommand is in, by the name `--help` prints for it.
    pub fn of(subcommand: &str) -> Class {
        CLASSIFIED
            .iter()
            .find(|(name, _)| *name == subcommand)
            .map_or(Class::Mutating, |(_, class)| *class)
    }
}

/// Every subcommand this table names, with its class.
///
/// **Data rather than a `match`, and that is the whole point of the shape.** The gate that keeps
/// these names honest — `tests/core/latency/latency_budget_tests.rs` — asserts them against `--help`, and a
/// `match` gives it nothing to iterate, so it read a hand-typed array of twenty-four strings
/// beside it instead. The copy omitted two entries, and one of them was `outdated`: a name
/// classified here that is not a subcommand at all (`shall list --outdated` is a flag), so the
/// arm was dead and the gate written to catch exactly that could not see it.
///
/// **The failure the gate guarded against was the failure it demonstrated** — `undo` sat in two
/// harness exemption lists because nothing validated the list, and the cure validated a
/// transcription of the list. A name that stops existing now fails that test, because the test
/// reads this.
///
/// Scanned linearly, and that is not worth a map: this is consulted once per command invocation.
const CLASSIFIED: &[(&str, Class)] = &[
    // Reads files, answers, stops.
    ("policy", Class::ConfigOnly),
    ("vars", Class::ConfigOnly),
    ("eval", Class::ConfigOnly),
    ("why", Class::ConfigOnly),
    ("protected", Class::ConfigOnly),
    ("completions", Class::ConfigOnly),
    ("path", Class::ConfigOnly),
    ("history", Class::ConfigOnly),
    ("diff", Class::ConfigOnly),
    ("plan", Class::ConfigOnly),
    ("profile", Class::ConfigOnly),
    ("module", Class::ConfigOnly),
    ("edit", Class::ConfigOnly),
    ("config", Class::ConfigOnly),
    ("hooks", Class::ConfigOnly),
    ("schedule", Class::ConfigOnly),
    ("fleet", Class::ConfigOnly),
    ("help", Class::ConfigOnly),
    ("info", Class::OneBackend),
    // **`sbom` and `export` are here, not in `ConfigOnly`.** They were filed as "reads files,
    // answers, stops" and they spawn one child process per manager — so the shape gate never
    // looked at them while they ran a serial loop at 1.0× overlap and twenty-one waves over
    // twenty-one children, and the five-second config budget they were handed printed a `WARN`
    // on an ordinary run. A class is about what a command *does*, not how it reads.
    ("sbom", Class::EveryBackend),
    ("export", Class::EveryBackend),
    ("list", Class::EveryBackend),
    ("search", Class::EveryBackend),
    ("check", Class::EveryBackend),
    ("adopt", Class::EveryBackend),
];

/// The names [`CLASSIFIED`] classifies, for the gate that checks them against `--help`.
///
/// Exists so that gate reads the table rather than a copy of it.
pub fn classified_names() -> impl Iterator<Item = &'static str> {
    CLASSIFIED.iter().map(|(name, _)| *name)
}

/// The subcommand name clap prints, taken off the `Commands` variant's own `Debug`.
///
/// Derived rather than listed: a second table of sixty-six names beside the enum is the shape
/// that produced an exemption for `undo`, a subcommand renamed away, sitting in two harness
/// lists for months. clap's rule is kebab-case of the variant name — `SelfUpgrade` is
/// `self-upgrade` — so the conversion is the same one clap made, in the other direction.
pub fn subcommand_name(command: &impl std::fmt::Debug) -> String {
    let debug = format!("{:?}", command);
    let variant = debug
        .split(|c: char| !c.is_ascii_alphanumeric())
        .next()
        .unwrap_or("");
    let mut out = String::with_capacity(variant.len() + 2);
    for (i, c) in variant.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                out.push('-');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// Say how long a command took, when it took longer than its class allows.
///
/// Reported rather than refused: a slow answer is still the answer, and a program that aborted
/// a `check` at five seconds would have turned a performance defect into a correctness one. The
/// point is that the number reaches somebody — E14 shipped because nobody was counting.
pub fn report_if_over(subcommand: &str, elapsed: Duration) {
    let class = Class::of(subcommand);
    report_shape(subcommand, class);
    let Some(budget) = class.budget() else {
        unmeasured(
            subcommand,
            "a wall-clock budget is the host's cost to answer, not Shall's, so this class carries \
             none — the shape and scheduling rules are what it is measured by instead",
        );
        return;
    };
    if elapsed <= budget {
        return;
    }
    tracing::warn!(
        "`shall {}` took {:.1}s. A {} command is budgeted {}s — this one is over, which is the \
         shape of the 98-second `info` that could not be seen because nothing measured it.",
        subcommand,
        elapsed.as_secs_f64(),
        match class {
            Class::ConfigOnly => "config-only",
            Class::OneBackend => "single-backend",
            _ => "read",
        },
        budget.as_secs(),
    );
}

/// Why a fan-out's shape is out of budget, or `None` if it is fine or unmeasurable.
///
/// Pure, and separate from the reporting, so the rule can be asserted without a package manager
/// and without a clock. Every argument comes from `core::timing`, which already computes all of
/// them for `--timings`.
pub fn shape_violation(
    shape: Shape,
    children: usize,
    summed: Duration,
    slowest: Duration,
    wall: Duration,
    waves: usize,
) -> Option<String> {
    let overlap = summed.as_secs_f64() / wall.as_secs_f64().max(f64::EPSILON);
    let mut faults = Vec::new();
    // **The gap suppresses this half only, and not by returning early.** The wave half below does
    // not depend on how long a child took, so a serial fan-out on a skewed host — the one case the
    // ratio cannot judge — is still caught by it. Returning early from the whole function on a gap
    // is what the first draft did, and its own test caught it: a fully serial run has
    // `slowest == summed`, so it claimed the skew exemption and went unreported.
    if shape_measurement_gap(shape, children, summed, slowest, waves).is_none()
        && overlap < shape.min_overlap
    {
        faults.push(format!(
            "{:.1}x overlap, under the {:.1}x floor — {} child command(s) summing to {:.2}s ran \
             in {:.2}s of wall clock, which is close to running them one at a time",
            overlap,
            shape.min_overlap,
            children,
            summed.as_secs_f64(),
            wall.as_secs_f64()
        ));
    }
    let ceiling = shape.wave_ceiling(children);
    if waves > ceiling {
        faults.push(format!(
            "{} wave(s) over {} child command(s), against a ceiling of {} — the run went quiet \
             {} time(s), because something had to be answered before the next question could be \
             asked, and that is close to asking them one at a time",
            waves,
            children,
            ceiling,
            waves - 1
        ));
    }
    (!faults.is_empty()).then(|| faults.join("; "))
}

/// Why a plan's execution went quiet more often than its dependencies required, or `None`.
///
/// The bound for [`Class::Mutating`], and the reason that class needs no [`Shape`]: a run cannot
/// go idle more times than its plan has levels. One wave per level is a perfect scheduler; one
/// wave per package is a serial loop wearing a graph. Anything above `depth` is the engine
/// waiting for work it had already been handed.
///
/// **A wave is an idle restart, not a dispatch.** Counting dispatches instead looks equivalent
/// and inverts the rule: two independent chains finishing at different moments produce a
/// dispatch per completion, which is eager scheduling doing its job, and the rule would have
/// failed the runs it exists to reward. Overlapping chains put waves *below* depth, so this is
/// an inequality in one direction only.
///
/// **Exact rather than tuned, which is what the fan-out rule could not be.** `min_overlap` and
/// `waves_per_child` are numbers read off four hosts, so they carry a margin and cannot be
/// tightened without a fifth reading. This one has no margin to pick: `waves > depth` is false
/// for every correct run on every host, because both quantities come from the same graph.
///
/// A single node has nothing to schedule, so plans under two packages are not measured — not
/// for noise, but because `waves > depth` is unreachable there and a rule that cannot fail is
/// what this replaced.
/// Whether a plan is big enough for the scheduling rule to be a measurement of anything.
///
/// One condition, two callers, for the reason [`Shape::is_measurable`] gives: the rule and the
/// reporter must not be able to disagree about when it is in force.
pub fn scheduling_is_measurable(packages: usize, depth: usize) -> bool {
    packages >= 2 && depth > 0
}

/// The one token every "we chose not to check this" line carries, and the reason the marker
/// cannot be renamed by accident: a build log is a thing people grep, and a token that changed
/// shape would break every grep anyone had already written without any of them finding out.
///
/// `const` rather than a literal at the call site so a test can hold the string the build log
/// will be searched for, rather than a copy of it.
pub const UNMEASURED_TOKEN: &str = "shall-latency-unmeasured:";

/// A single site said *we chose not to check this*, on a channel that does not cost a user
/// attention, carrying a token that a build can count.
///
/// **The gap this closes is the one II.23 names at the other end: a scan that reaches nothing
/// reports nothing, which reads exactly like a clean tree.** Every function above has a way to
/// decline, and each of those ways used to be `return` and nothing else — so a run with no budget
/// for its class, a fan-out too small to have an overlap ratio, a plan with one package, and a
/// command that never enabled its own timings were four different situations that all printed
/// nothing and all looked identical to a reader. *"We decided this is not measurable"* and
/// *"nobody is measuring this"* are different sentences and the output said neither.
///
/// One token rather than four, so the question has one answer: `grep -c shall-latency-unmeasured:`
/// on a build log is how many measurements were declined, and the text after the colon is which.
///
/// **At `info`, and that placement is a ruling, not an accident** (owner, 2026-09-29). The default
/// level is `warn` and these fire on most commands, so at `warn` a user who typed a package name
/// would be shown a line about a budget that does not exist — which is not information for them
/// and is the reason such warnings get filtered into `/dev/null` along with the real ones. Below
/// the default the marker is invisible unless somebody asked (`-v`, or `RUST_LOG=info`), and the
/// thing that actually enforces it is `unmeasured_is_reported_on_every_skip_path`, which drives
/// each path and fails if the marker is absent. **A grep is a convenience; the gate is the
/// guarantee.**
pub fn unmeasured(subject: &str, why: &str) {
    tracing::info!("{UNMEASURED_TOKEN} {subject} — {why}");
}

pub fn scheduling_violation(packages: usize, depth: usize, waves: usize) -> Option<String> {
    if !scheduling_is_measurable(packages, depth) || waves <= depth {
        return None;
    }
    Some(format!(
        "{} wave(s) over {} package(s) whose longest dependency chain is {} — the engine ran dry \
         and waited {} more time(s) than the plan required, which is what this loop looks like \
         just after somebody makes it await the batch in flight",
        waves,
        packages,
        depth,
        waves - depth
    ))
}

/// Say when a fan-out stopped fanning out.
///
/// Only on a run that asked for `--timings`, because that is the only run that records spans.
/// That is a real limit and it is stated rather than papered over: the *gate* is
/// `tests/core/latency/latency_budget_tests.rs`, which drives the fan-out commands with `--timings` on
/// purpose. This is what puts the same sentence in front of a user who asked.
/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
/// crate links the library without `cfg(test)`, so a private item is invisible there.
pub fn report_shape(subcommand: &str, class: Class) {
    let Some(shape) = class.shape() else {
        unmeasured(
            subcommand,
            "no fan-out shape budget for this class — a single backend or a config-only command \
             has nothing to overlap, and a mutating run is measured by the plan's own depth \
             instead",
        );
        return;
    };
    if !crate::core::timing::is_enabled() {
        unmeasured(
            subcommand,
            "this run did not pass `--timings`, so no spans were recorded and the overlap ratio \
             was never computed",
        );
        return;
    }
    let (rows, _, summed) = crate::core::timing::summary();
    let children: usize = rows.iter().map(|r| r.calls).sum();
    // The slowest single child, which is what pins the ratio's ceiling. A `Row`'s `longest` is
    // its own label's worst call, so the max over rows is the slowest call anywhere in the run.
    let slowest = rows.iter().map(|r| r.longest).max().unwrap_or_default();
    let (waves, width) = crate::core::timing::schedule();
    // **A run that was more than one wide overlapped, and neither of this rule's numbers can
    // judge it from there.** Both were calibrated on a host whose `max_parallel` was large enough
    // to hide every child inside the slowest one; a two-slot budget over five managers with a
    // `list` and a `prefix` each is capped at 1.1x overlap and six waves by arithmetic, not by
    // scheduling. Measured on the CI runners, same code, same thresholds: Windows 7 children
    // summing to 5.23s inside 4.25s (1.2x, 4 waves) and Linux 7 summing to 1.89s inside 1.68s
    // (1.1x, 6 waves) — both with two children demonstrably in flight together. Reporting those
    // as a collapse is the warning-every-time failure this file's neighbours keep naming.
    //
    // The reading that does not move with the host's budget is the width, and it is exact: a
    // `for` loop awaiting each child is 1 wide whatever a child costs. So a one-wide run is
    // reported as the collapse it is, and a two-wide run is recorded as unmeasured at `info`
    // rather than shouted about at `warn`.
    //
    // **What this gives up is stated rather than buried.** A fan-out that uses two slots where
    // it had sixteen is no longer reported by the ratio, because the ratio cannot tell that run
    // from one on a host with two slots. The gates carry it instead and always did: the ratio
    // and the wave ceiling are asserted directly for `list` by `tests/core/latency/latency_budget_tests.rs`,
    // and the width for these two by `tests/core/latency/the_other_fan_outs_fan_out_too_tests.rs`.
    if width >= 2 {
        unmeasured(
            subcommand,
            &format!(
                "{width} child command(s) were in flight at once, so this fan-out overlapped; \
                 the ratio and the wave ceiling are bounded by the run's own `max_parallel` and \
                 child count rather than by the scheduler, and the width is the reading they \
                 cannot supply"
            ),
        );
        return;
    }
    if let Some(why) = shape_measurement_gap(shape, children, summed, slowest, waves) {
        unmeasured(subcommand, &why);
        // **Still judge the wave half**, because `shape_violation` gates only the overlap fault on
        // the gap. Reporting "not measured" and then not measuring would leave a serialised fan-out
        // on a skewed host with nothing said about it — which is the run the sentence above admits
        // it cannot judge.
        if let Some(why) = shape_violation(
            shape,
            children,
            summed,
            slowest,
            crate::core::timing::elapsed(),
            waves,
        ) {
            tracing::warn!(
                "`shall {}` did not overlap every manager: {}. The seconds a fan-out costs belong \
                 to the host; the scheduling does not.",
                subcommand,
                why
            );
        }
        return;
    }
    // `None` from here is a PASS, not a skip, and says nothing: the rule ran and the run was
    // within it. That is the distinction this whole mechanism exists to keep, so the skip above
    // is checked first rather than being folded into this one `None`.
    let Some(why) = shape_violation(
        shape,
        children,
        summed,
        slowest,
        crate::core::timing::elapsed(),
        waves,
    ) else {
        return;
    };
    tracing::warn!(
        "`shall {}` asked every manager and did not overlap them: {}. The seconds a fan-out \
         costs belong to the host; the scheduling does not.",
        subcommand,
        why
    );
}
