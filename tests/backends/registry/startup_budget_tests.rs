//! What a command costs before it has done anything.
//!
//! `shall path` measured **272 ms** on a release build against a 61 ms process-spawn baseline,
//! and `--timings` said `no child commands — this run asked no package manager anything`. All of
//! it was fixed overhead: `create_default_registry` runs for every subcommand, and one backend
//! built a rate limiter in its constructor whose clock performs a 200 ms TSC calibration on
//! first construction (AU3).
//!
//! The reason it survived is the one `AU3` states plainly: **nothing measured the part of a run
//! that spawns no child.** `latency.rs` budgets a whole command at seconds, which a fifth of a
//! second of start-up never crosses, and every per-backend timing instrument measures child
//! processes — of which this has none.
//!
//! So this file budgets the two halves separately:
//!
//! * a limit on building the registry, **measured against this machine's own pace** rather than
//!   against a fixed millisecond count — the class, *any* eagerly constructed expensive object
//!   lands here whatever the cause, with the host's speed divided out (see the constant);
//! * the invariant behind the specific one, asserted deterministically: a rate limiter costs
//!   nothing until something is rate limited.

use shall::backends::create_default_registry;
use std::io::Write;
use std::process::Stdio;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// The budget, **as a multiple of what this machine costs to do nothing** (`V.233`, `II.79`).
///
/// A fixed millisecond ceiling measured the host, not the product. This file went red in two
/// consecutive full-suite runs at 145.8 ms and 148.4 ms against 120 ms, in suites that themselves
/// took 480–530 s instead of the usual 260 s, while passing three isolated runs in a row at
/// ~55 ms — nothing about the registry had changed between them. That is `V.231` in its most
/// literal form: a test asserting the host, which is a gate people learn to skip.
///
/// So the child times a fixed, trivial workload of its own first, and registration is judged
/// against a multiple of *that*. The property "registration constructs nothing eagerly" survives
/// intact — a backend that builds an object in its constructor still blows through it, because
/// the work it adds is unrelated to the baseline — and the machine's clock speed divides out.
///
/// The multiple is calibrated from the measurements that found the flake: ~55 ms of registration
/// against a baseline of the same order on an idle machine, and ~148 ms under the load that also
/// doubled a suite's wall time, which inflates the baseline by roughly the same factor.
const REGISTRY_BUDGET_IN_BASELINES: u32 = 12;
const CHILD_ENV: &str = "SHALL_REGISTRY_BUDGET_CHILD";
const CHILD_DELAY_ENV: &str = "SHALL_REGISTRY_BUDGET_DELAY_MS";
const DURATION_PREFIX: &str = "SHALL_REGISTRY_BUDGET_US=";
const BASELINE_PREFIX: &str = "SHALL_REGISTRY_BASELINE_US=";
const BACKENDS_PREFIX: &str = "SHALL_REGISTRY_BACKENDS=";
const MEASUREMENT_TEST: &str =
    "building_the_registry_asks_nothing_and_therefore_costs_almost_nothing";

#[tokio::test]
async fn building_the_registry_asks_nothing_and_therefore_costs_almost_nothing() {
    if std::env::var_os(CHILD_ENV).is_some() {
        measure_registry_in_this_process().await;
        return;
    }

    let mut measurements = Vec::new();
    for _ in 0..3 {
        measurements.push(measure_registry_in_a_fresh_process(None).await);
    }
    // The *ratio* is the judgement, and the fastest run of the three decides it — a loaded
    // machine inflates both halves of the fraction, which is the whole point of having one.
    let (best, best_baseline, backends) = measurements
        .iter()
        .min_by(|a, b| ratio_of(a.0, a.1).total_cmp(&ratio_of(b.0, b.1)))
        .copied()
        .expect("three registry measurements");
    let ratio = ratio_of(best, best_baseline);
    let budget = REGISTRY_BUDGET_IN_BASELINES as f64;
    assert!(
        ratio <= budget,
        "registering {backends} backends took {best:.1?} against a {best_baseline:.1?} baseline \
         — {ratio:.1}x the machine's own pace, over the {budget:.0}x budget. Both numbers come \
         from the same process, so a loaded machine inflates them together and cannot fail this.\n\
         Registration runs for every subcommand and asks no manager anything, so this is fixed \
         overhead on `shall path` as much as on `shall sync`. Something is being CONSTRUCTED \
         eagerly that is only USED conditionally — build it where it is used, as `web.rs` and \
         `appimage.rs` build their HTTP clients.",
    );

    // **The control is judged against its own baseline, in its own process.** A delay injected
    // into one child and compared against the ratio of an earlier child measures two processes
    // on a machine whose load moved between them — the first version of this compared the
    // control's cost with the *passing* run's baseline, and read `28.1ms against 85.6ms` for a
    // process that had deliberately slept 240 ms.
    // Generous enough to clear the budget by a wide margin whatever the machine's pace: the
    // budget is a ratio, so the control has to be expressed in the same unit.
    let control_delay =
        (best_baseline.as_millis() as u64) * (REGISTRY_BUDGET_IN_BASELINES as u64 * 3);
    let (control, control_baseline, _) =
        measure_registry_in_a_fresh_process(Some(control_delay)).await;
    let control_ratio = duration_ratio(control, control_baseline);
    assert!(
        control_ratio > (REGISTRY_BUDGET_IN_BASELINES as f64) * 1.5,
        "a fresh process that spent {control_delay} ms constructing the registry measured \
         {control:.1?} against its own {control_baseline:.1?} baseline — {control_ratio:.1}x, \
         which is not over the {budget:.0}x budget — so the ratio cannot detect an over-budget \
         construction at all"
    );
}

/// How long registration cost, how long this machine's own pace was, and how many backends.
async fn measure_registry_in_a_fresh_process(delay_ms: Option<u64>) -> (Duration, Duration, usize) {
    let exe = std::env::current_exe().expect("the current test executable");
    let module = module_path!()
        .split_once("::")
        .map(|(_, path)| path)
        .unwrap_or(module_path!());
    let test_name = format!("{module}::{MEASUREMENT_TEST}");
    let mut child = tokio::process::Command::new(exe);
    child
        .arg("--exact")
        .arg(test_name)
        .arg("--nocapture")
        .env(CHILD_ENV, "1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(delay_ms) = delay_ms {
        child.env(CHILD_DELAY_ENV, delay_ms.to_string());
    }
    let output = child
        .output()
        .await
        .expect("the registry measurement child");
    assert!(
        output.status.success(),
        "the registry measurement child failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    let stdout = String::from_utf8(output.stdout).expect("the measurement output is UTF-8");
    let duration = stdout
        .lines()
        .find_map(|line| line.trim().strip_prefix(DURATION_PREFIX))
        .and_then(|value| value.parse::<u64>().ok())
        .map(Duration::from_micros)
        .unwrap_or_else(|| panic!("the measurement child printed no duration:\n{stdout}"));
    let baseline = stdout
        .lines()
        .find_map(|line| line.trim().strip_prefix(BASELINE_PREFIX))
        .and_then(|value| value.parse::<u64>().ok())
        .map(Duration::from_micros)
        .unwrap_or_else(|| panic!("the measurement child printed no baseline:\n{stdout}"));
    let backends = stdout
        .lines()
        .find_map(|line| line.trim().strip_prefix(BACKENDS_PREFIX))
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or_else(|| panic!("the measurement child printed no backend count:\n{stdout}"));
    (duration, baseline, backends)
}

/// `cost / baseline`, guarded against a zero baseline.
fn ratio_of(cost: Duration, baseline: Duration) -> f64 {
    duration_ratio(cost, baseline)
}

fn duration_ratio(cost: Duration, baseline: Duration) -> f64 {
    cost.as_secs_f64() / baseline.as_secs_f64().max(f64::MIN_POSITIVE)
}

async fn measure_registry_in_this_process() {
    let vfs = Arc::new(dashmap::DashMap::new());
    let mock = Arc::new(shall::core::executor::MockExecutor::new(vfs.clone()));
    let exec = shall::core::CommandExecutor::with_layer(
        true,
        false,
        mock,
        vfs,
        Arc::new(dashmap::DashMap::new()),
    );
    let config = shall::config::Config::default();
    let hooks = Arc::new(shall::app::hooks::LuaHooks::new(&config).expect("hooks"));

    // The baseline: the best of three runs of work this machine can be timed doing that has
    // nothing to do with the registry. Taken here rather than in the parent because the parent
    // is not the process being judged — this one is, and a baseline measured anywhere else would
    // be measuring a different amount of contention.
    let baseline = {
        let mut best = Duration::MAX;
        for _ in 0..3 {
            let started = Instant::now();
            std::hint::black_box(calibration_work());
            best = best.min(started.elapsed());
        }
        best
    };

    // **The control, inside the window.** It stands in for the thing this file is about — an
    // expensive object built eagerly during registration — so it has to be spent inside the
    // measurement, not before it. A `std::thread::sleep` rather than `tokio::time::sleep`
    // because under `#[tokio::test]` the runtime has no other task to run, the clock
    // auto-advances, and the sleep returns at once: the first version of this control spent a
    // nominal 240 ms and measured 82.5 ms, which was the sleep being skipped rather than the
    // ratio being blind.
    let injected = std::env::var(CHILD_DELAY_ENV)
        .ok()
        .and_then(|ms| ms.parse::<u64>().ok())
        .map(Duration::from_millis);

    let started = Instant::now();
    let registry = create_default_registry(exec, &config, hooks).await;
    if let Some(injected) = injected {
        std::thread::sleep(injected);
    }
    let elapsed = started.elapsed();
    assert!(!registry.all().is_empty());

    println!("{DURATION_PREFIX}{}", elapsed.as_micros());
    println!("{BASELINE_PREFIX}{}", baseline.as_micros().max(1));
    println!("{BACKENDS_PREFIX}{}", registry.all().len());
    std::io::stdout().flush().expect("the measurement output");
}

/// Work whose cost tracks the machine, used only as a divisor.
///
/// Deliberately not `Registry::new()`: the registry is very nearly the thing under test, so a
/// baseline built from it would shrink when the regression it is meant to catch appeared. This is
/// allocation plus integer arithmetic — nothing the registry influences and nothing that can be
/// optimised away, since `black_box` keeps the result alive.
fn calibration_work() -> u64 {
    let mut acc: u64 = 0;
    for i in 0..2_000_000u64 {
        acc = acc.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(i);
        let v = acc >> 33;
        acc ^= v.rotate_left(17);
    }
    acc
}

#[tokio::test]
async fn a_rate_limiter_costs_nothing_until_something_is_rate_limited() {
    // Every constructor on the type, not just the one that was found eager: the defect was a
    // backend calling `RateLimiter::github()` in `new`, and the sibling call site
    // (`vscode.rs`) does exactly the same thing.
    for limiter in [
        shall::core::RateLimiter::github(),
        shall::core::RateLimiter::github_authenticated(),
        shall::core::RateLimiter::vscode_marketplace(),
        shall::core::RateLimiter::new(10, "test"),
    ] {
        assert!(
            !limiter.is_engaged(),
            "`{}` built its limiter in the constructor. Nothing has been rate limited yet, so \
             nothing should have been built — a backend's `new` runs for every subcommand.",
            limiter.description()
        );
        limiter.wait().await.expect("first permit is immediate");
        assert!(
            limiter.is_engaged(),
            "`{}` waited for a permit without a limiter to issue one",
            limiter.description()
        );
    }
}

/// The clones share one limiter, which is the whole point of rate limiting: two backends holding
/// copies of the same quota must not get one quota each. Laziness is easy to add in a way that
/// quietly breaks this — a per-clone cell would.
#[tokio::test]
async fn clones_share_the_quota_they_were_cloned_from() {
    let original = shall::core::RateLimiter::new(5, "shared");
    let clone = original.clone();

    clone.wait().await.expect("first permit is immediate");

    assert!(
        original.is_engaged(),
        "the clone built its own limiter, so the two hold separate quotas and the limit is \
         twice what it says it is"
    );
}
