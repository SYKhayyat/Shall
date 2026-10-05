//! `sbom` and `export` must ask their managers at the same time, and this measures that with no
//! package managers installed at all.
//!
//! **Filed as #107**, from two suite failures that read like the skew in `PLAN.md` #90 and are
//! not it. On a seven-manager host:
//!
//! ```text
//! `shall sbom` asked 7 managers and overlapped them 1.1x, under the 1.2x floor
//!   Timings: 7.93s wall · 7 child command(s) summing to 8.67s · 1.1x overlap · 6 wave(s)
//! ```
//!
//! Six waves over seven children is close to one at a time. #90 was a *one*-wave run on a skewed
//! host, where the low ratio was the ratio's ceiling rather than a scheduler — and #90's fix
//! deliberately withholds its skew exemption above one wave precisely so a serial run is still
//! judged. These were judged, and they failed, correctly.
//!
//! **So this file asks the question directly, and it can be asked anywhere.** Seven mock backends
//! each answer `list_installed` after a fixed delay, and the wall clock is the answer: seven that
//! overlap take about one delay, and seven that do not take seven. No host needs four real
//! managers, no network is involved, and the number that decides it is a ratio rather than a clock
//! reading — so neither a fast machine nor a slow one decides the result.
//!
//! **What it does not claim.** It does not say the product is wrong; it says the question is now
//! answerable on any machine, which is what the development host was not. One manager is below the
//! floor's own `min_children` of four, so `the_other_fan_outs_fan_out_too_tests` skips there and
//! proves nothing locally. If this test is green, the serialisation the CI run saw is not in
//! `managed_pkgs`, and #107's remaining question is the one its own issue names — *which seven
//! commands those were*, and whether the fan-out was what the harness timed at all.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use shall::backends::generic::{
    GenericBackendCore, GenericQueryable, ManagerConfig, ManualListing,
};
use shall::backends::BackendRegistry;
use shall::core::executor::{CommandExecutor, DryRunOutput, MockExecutor};
use shall::core::BackendCapabilities;
use shall::core::{ManagedPackage, StateRegistry};

/// How many mock managers to fan out over. Four is the floor's own `min_children`, so a run that
/// overlapped nothing here is one the shipped gate is entitled to judge.
const MANAGERS: usize = 7;

/// Each backend's listing costs this long — long enough that seven serial answers cannot be
/// mistaken for seven parallel ones on a loaded machine, and short enough that a serial run
/// finishes well under a second.
const COST: Duration = Duration::from_millis(120);

fn managed(backend: &str, name: &str) -> ManagedPackage {
    ManagedPackage {
        name: name.to_string(),
        backend: backend.to_string(),
        version: Some("1.0".into()),
        installed_at: 0,
        expires_at: None,
        options: Default::default(),
        source: "a test".into(),
        is_transient: false,
        session_id: None,
    }
}

/// A registry of `count` queryable mock backends, each answering its listing after `COST`.
///
/// **The delay is what makes the measurement.** `MockExecutor`'s delay is a real
/// `tokio::time::sleep` inside `execute`, so two of them at once are two overlapping tasks and the
/// wall clock can tell the difference. An instantaneous mock makes serial and parallel
/// indistinguishable, which is the failure the mock's own comment on `set_delay` warns about.
/// **A counting wrapper around the mock, so "did these overlap" is answered by a count.**
///
/// Elapsed time measures the machine as well as the code. Two runs of the fan-out test below
/// failed on this host at 875ms and then at 1.6s against an 840ms serial baseline — nothing but
/// an unrelated `--release` build on the same host, load average 33 — and passed at 0.28s when it
/// was quiet. A serial run and a broken instrument looked identical, and the serial run was the
/// thing under test.
///
/// The counter lives here rather than in `MockExecutor` because it is a test-only concern, and
/// because `executor.rs` is at its recorded line ceiling: the first attempt added the counting to
/// the mock itself and `a_module_is_a_subject_not_a_pile_tests` refused it at 3223 lines against a
/// 3150 ceiling. Wrapping the layer costs `src/` nothing and keeps the instrument next to the
/// assertions that read it.
struct Counting {
    inner: Arc<MockExecutor>,
    seen: Arc<tokio::sync::Mutex<Peak>>,
}

#[derive(Default, Clone)]
struct Peak {
    in_flight: usize,
    peak: usize,
    started: usize,
    at: Vec<String>,
}

#[async_trait::async_trait]
impl shall::core::executor::ExecutionLayer for Counting {
    async fn execute(
        &self,
        cmd: &str,
        args: &[String],
        env: &std::collections::HashMap<String, String>,
    ) -> shall::core::Result<std::process::Output> {
        {
            let mut p = self.seen.lock().await;
            p.in_flight += 1;
            p.started += 1;
            if p.in_flight > p.peak {
                p.peak = p.in_flight;
                p.at = self
                    .inner
                    .call_log
                    .lock()
                    .await
                    .iter()
                    .rev()
                    .take(p.in_flight)
                    .cloned()
                    .collect();
            }
        }
        let out = self.inner.execute(cmd, args, env).await;
        self.seen.lock().await.in_flight -= 1;
        out
    }

    fn check_command(&self, cmd: &str) -> bool {
        self.inner.check_command(cmd)
    }

    async fn symlink(
        &self,
        src: &std::path::Path,
        dst: &std::path::Path,
    ) -> shall::core::Result<()> {
        self.inner.symlink(src, dst).await
    }
}

/// The same registry, with the mock kept so a test can read the call log.
///
/// **The call log, not a clock.** Every timing assertion in this file can be moved by another
/// process on the same machine — one of them was, by a `--release` build — and the question
/// "was this manager asked once or once per package" is not a question about elapsed time at
/// all. Counting the commands is the whole measurement.
fn registry_and_mock_of(
    count: usize,
) -> (
    Arc<BackendRegistry>,
    Arc<MockExecutor>,
    Arc<tokio::sync::Mutex<Peak>>,
) {
    let vfs: Arc<DashMap<PathBuf, String>> = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    // The layer is the counting wrapper; the mock underneath still records the call log, which is
    // where the peak's "which commands met" list comes from.
    let peak = Arc::new(tokio::sync::Mutex::new(Peak::default()));
    let layer = Arc::new(Counting {
        inner: mock.clone(),
        seen: peak.clone(),
    });
    let exec = CommandExecutor::with_layer(true, false, layer, vfs, Arc::new(DashMap::new()));
    let mut reg = BackendRegistry::new();
    for i in 0..count {
        let name = format!("mock{i}");
        mock.set_command_exists(&name, true);
        // `MockExecutor` matches on program plus argv, and this is the call that has to overlap:
        // `info` reaches the listing through `installed_listing`, one fetch per backend.
        mock.set_delay(&format!("{name} list"), COST);
        // **`installed <name> <version>`, the row `AptParser::read_row` reads, and not a
        // shorthand.** The fixture used to answer `{name}-pkg 1.0`, which `read_row` rejects —
        // a row must open with a status word dpkg can actually emit — and an unreadable row is
        // not an empty machine, it is an **error**.
        //
        // Which matters because `once` deliberately does not cache a failure: *"a manager that
        // could not answer this time may answer next time"*. Every fetch was therefore erroring,
        // the memo never filled, and every lookup re-ran its manager. **Every fan-out
        // measurement in this file was taken on the error path** — the widths and durations
        // recorded were those of repeated failures rather than of a fan-out, and the memo this
        // file's tests are about was never exercised at all.
        mock.set_response(
            &format!("{name} list"),
            Ok(DryRunOutput {
                stdout: format!("installed  {name}-pkg  1.0\n").into_bytes(),
                stderr: vec![],
            }
            .into()),
        );
        let config = ManagerConfig {
            name: name.clone(),
            binary: None,
            remove_binary: None,
            install_args: vec![],
            remove_args: vec![],
            list_args: vec!["list".into()],
            list_binary: None,
            manual: ManualListing::AllInstalled,
            essential_args: None,
            search_args: vec![],
            search_binary: None,
            enumerate_args: None,
            enumerate_binary: None,
            upgrade_args: vec![],
            update_args: None,
            purge_args: None,
            orphan_dry_run: None,
            foreign_args: None,
            repo_add_args: None,
            repo_remove_args: None,
            repo_list_args: None,
            repo_binary: None,
            repo_list_binary: None,
            repo_remove_binary: None,
            repo_list_shape: shall::backends::generic::RepoListing::Columns,
            depends: None,
            clean_cache: None,
            version_pin: None,
            needs_root: false,
            is_exclusive: false,
            install_source_option: None,
            extra_probes: None,
            upgrade_reinstall_args: None,
            property_probes: Vec::new(),
            machine_list: None,
            outdated: None,
            search_source: shall::backends::generic::SearchSource::Command,
            qualified_names: false,
        };
        let core = Arc::new(GenericBackendCore {
            name,
            executor: exec.clone(),
            config,
            parser: Arc::new(shall::parsers::apt::AptParser),
        });
        reg.register(Arc::new(
            BackendCapabilities::builder(core.clone())
                .with_queryable(Arc::new(GenericQueryable { core }))
                .build(),
        ));
    }
    (Arc::new(reg), mock, peak)
}

/// Seven managers, one managed package each — the state that makes `managed_pkgs` fan out at all.
/// On an empty registry there is nothing to ask anyone, which is the question #107 raises first.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn seven_managers_are_asked_at_once_and_not_one_after_another() {
    let (registry, _mock, peak) = registry_and_mock_of(MANAGERS);
    let state = Arc::new(tokio::sync::Mutex::new(StateRegistry::default()));
    {
        let mut guard = state.lock().await;
        guard.set_managed(
            (0..MANAGERS).map(|i| managed(&format!("mock{i}"), &format!("mock{i}-pkg"))),
        );
    }

    let pkgs = shall::app::export::managed_pkgs(&state, &registry, 4).await;

    assert_eq!(
        pkgs.len(),
        MANAGERS,
        "every managed package should have been answered, so the peak below is about a fan-out \
         and not about a short list"
    );

    // **Peak concurrency, not elapsed time.** This assertion used to be `elapsed < serial / 2`,
    // and it failed on this host at 875ms and then at 1.6s against an 840ms serial baseline —
    // both times purely because an unrelated `--release` build was running (load average 33),
    // and both times passing at 0.28s when the machine was quiet. A clock measures the host as
    // well as the code. The peak is the number of managers that were *simultaneously* being
    // asked, which is what the claim is about and which no amount of load can move.
    let c = peak.lock().await.clone();
    assert!(
        c.peak > 1,
        "{MANAGERS} managers each cost {COST:?}, and at most {} were ever in flight at once (of \
         {} commands) — so they were asked one at a time. This is #107: `shall sbom` and \
         `shall export` measured 1.1x over 6 waves on a seven-manager host, and no host with \
         four managers could reproduce it until this test.",
        c.peak,
        c.started
    );
    assert!(
        c.peak >= 4,
        "the fan-out was asked for a width of 4 and reached {}: {:?}",
        c.peak,
        c.at
    );
}

/// **Does one manager get asked once, or once per package? Counted, not timed.**
///
/// This is the narrow question `#107` narrowed to. Its own doc on
/// [`Queryable::list_installed`] claims the memo means a run gets here once per manager, and
/// `fetch_installed`'s comment says the same — "costs one invocation per run, because the listing
/// memo means a run gets here once per manager". Twelve lookups spread over four managers
/// measured like **twelve** child commands rather than four, which is what put the memo under
/// suspicion in the first place.
///
/// The measurement is a count of recorded commands, because that is what the claim is about and
/// because a clock cannot answer it here: the timing assertions in this file are demonstrably
/// movable by another build on the same machine. No delay is set, so this test is not a
/// concurrency claim and cannot fail because the machine was busy.
#[tokio::test]
async fn a_manager_is_listed_once_however_many_packages_are_asked_about_it() {
    const BACKENDS: usize = 4;
    const PER_BACKEND: usize = 3;

    let (registry, mock, _peak) = registry_and_mock_of(BACKENDS);
    let state = Arc::new(tokio::sync::Mutex::new(StateRegistry::default()));
    {
        let mut guard = state.lock().await;
        guard.set_managed((0..BACKENDS).flat_map(|b| {
            (0..PER_BACKEND).map(move |i| managed(&format!("mock{b}"), &format!("mock{b}-pkg{i}")))
        }));
    }

    let pkgs = shall::app::export::managed_pkgs(&state, &registry, 2).await;
    assert_eq!(
        pkgs.len(),
        BACKENDS * PER_BACKEND,
        "every package must still be answered: the memo is an optimisation, never a reason to \
         return less"
    );

    let calls = mock.get_calls().await;
    let listings: Vec<&String> = calls.iter().filter(|c| c.ends_with("list")).collect();

    // One listing per manager is the whole claim. This is the number that decides whether
    // `installed_listing`'s per-backend singleflight is working at all, and it is exactly what a
    // wall clock could not tell us.
    assert_eq!(
        listings.len(),
        BACKENDS,
        "each of {BACKENDS} managers was asked {} times for {BACKENDS} packages of {} each: {:?}. \
         `installed_listing` holds the slot across the fetch so two askers produce one subprocess, \
         so this is the memo failing rather than the fan-out being narrow — and it makes the \
         fan-out's `buffered(max_parallel)` irrelevant, because every future is asking something \
         it already has the answer to.",
        listings.len(),
        PER_BACKEND,
        listings
    );
}

/// **The control, and it is what makes the timing above mean something.** The same seven backends
/// and the same seven packages, asked with a width of **one** — so the loop is explicit rather
/// than inferred, and the difference between this and the test above is the whole claim.
///
/// A width of one makes `buffered(1)` a serial loop, which is what a `max_parallel` of zero or a
/// caller that forgot the setting would produce. If this measured the same wall clock as the
/// fan-out, then the fan-out test above would be measuring something other than concurrency.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_width_of_one_is_serial_because_it_is_meant_to_be() {
    let (registry, _mock, peak) = registry_and_mock_of(MANAGERS);
    let state = Arc::new(tokio::sync::Mutex::new(StateRegistry::default()));
    {
        let mut guard = state.lock().await;
        guard.set_managed(
            (0..MANAGERS).map(|i| managed(&format!("mock{i}"), &format!("mock{i}-pkg"))),
        );
    }

    let pkgs = shall::app::export::managed_pkgs(&state, &registry, 1).await;

    assert_eq!(pkgs.len(), MANAGERS, "same seven packages");
    // **The control is the same instrument, not a clock.** It used to assert `elapsed >= 6 *
    // COST`, which passed on a loaded machine for the wrong reason — a busy host stretches every
    // delay, so a serial run and a broken instrument looked alike. A width of one means exactly
    // one command in flight, and the peak says so whether the machine took 3 seconds or 3
    // minutes.
    let c = peak.lock().await.clone();
    assert_eq!(
        c.peak, 1,
        "a width of one had {} commands in flight at once, of {} — so the counter cannot tell a \
         loop from a fan-out, and the test above measures nothing",
        c.peak, c.started
    );
}
