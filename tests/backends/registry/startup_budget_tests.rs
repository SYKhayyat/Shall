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
//! * a wall-clock ceiling on building the registry, which is the class — *any* eagerly
//!   constructed expensive object lands here, whatever the cause;
//! * the invariant behind the specific one, asserted deterministically: a rate limiter costs
//!   nothing until something is rate limited.

use shall::backends::create_default_registry;
use std::io::Write;
use std::process::Stdio;
use std::sync::Arc;
use std::time::{Duration, Instant};

const REGISTRY_BUDGET: Duration = Duration::from_millis(120);
const CHILD_ENV: &str = "SHALL_REGISTRY_BUDGET_CHILD";
const CHILD_DELAY_ENV: &str = "SHALL_REGISTRY_BUDGET_DELAY_MS";
const DURATION_PREFIX: &str = "SHALL_REGISTRY_BUDGET_US=";
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
    let (best, backends) = measurements
        .iter()
        .min_by_key(|measurement| measurement.0)
        .copied()
        .expect("three registry measurements");
    assert!(
        best <= REGISTRY_BUDGET,
        "registering {backends} backends took {best:.1?} in the best of three fresh processes, over the {REGISTRY_BUDGET:.1?} budget.\n\
         Registration runs for every subcommand and asks no manager anything, so this is fixed \
         overhead on `shall path` as much as on `shall sync`. Something is being CONSTRUCTED \
         eagerly that is only USED conditionally — build it where it is used, as `web.rs` and \
         `appimage.rs` build their HTTP clients.",
    );

    let control_delay = REGISTRY_BUDGET.as_millis() as u64 + 50;
    let (control, _) = measure_registry_in_a_fresh_process(Some(control_delay)).await;
    assert!(
        control > REGISTRY_BUDGET,
        "a fresh process that spent {control_delay} ms constructing the registry measured {control:.1?}, so the measurement cannot detect an over-budget construction"
    );
}

async fn measure_registry_in_a_fresh_process(delay_ms: Option<u64>) -> (Duration, usize) {
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
    let backends = stdout
        .lines()
        .find_map(|line| line.trim().strip_prefix(BACKENDS_PREFIX))
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or_else(|| panic!("the measurement child printed no backend count:\n{stdout}"));
    (duration, backends)
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

    let started = Instant::now();
    if let Ok(delay_ms) = std::env::var(CHILD_DELAY_ENV) {
        let delay_ms = delay_ms.parse::<u64>().expect("the measurement delay");
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    }
    let registry = create_default_registry(exec, &config, hooks).await;
    let elapsed = started.elapsed();
    assert!(!registry.all().is_empty());

    println!("{DURATION_PREFIX}{}", elapsed.as_micros());
    println!("{BACKENDS_PREFIX}{}", registry.all().len());
    std::io::stdout().flush().expect("the measurement output");
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
