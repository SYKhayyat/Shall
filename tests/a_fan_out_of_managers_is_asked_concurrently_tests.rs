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
use std::time::{Duration, Instant};

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
fn registry_of(count: usize) -> Arc<BackendRegistry> {
    let vfs: Arc<DashMap<PathBuf, String>> = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    let exec =
        CommandExecutor::with_layer(true, false, mock.clone(), vfs, Arc::new(DashMap::new()));
    let mut reg = BackendRegistry::new();
    for i in 0..count {
        let name = format!("mock{i}");
        mock.set_command_exists(&name, true);
        // `MockExecutor` matches on program plus argv, and this is the call that has to overlap:
        // `info` reaches the listing through `installed_listing`, one fetch per backend.
        mock.set_delay(&format!("{name} list"), COST);
        mock.set_response(
            &format!("{name} list"),
            Ok(DryRunOutput {
                stdout: format!("{name}-pkg 1.0\n").into_bytes(),
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
    Arc::new(reg)
}

/// Seven managers, one managed package each — the state that makes `managed_pkgs` fan out at all.
/// On an empty registry there is nothing to ask anyone, which is the question #107 raises first.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn seven_managers_are_asked_at_once_and_not_one_after_another() {
    let registry = registry_of(MANAGERS);
    let state = Arc::new(tokio::sync::Mutex::new(StateRegistry::default()));
    {
        let mut guard = state.lock().await;
        guard.set_managed(
            (0..MANAGERS).map(|i| managed(&format!("mock{i}"), &format!("mock{i}-pkg"))),
        );
    }

    let started = Instant::now();
    let pkgs = shall::app::export::managed_pkgs(&state, &registry, 4).await;
    let elapsed = started.elapsed();

    assert_eq!(
        pkgs.len(),
        MANAGERS,
        "every managed package should have been answered, so the timing below is about a fan-out \
         and not about a short list"
    );

    let serial = COST * MANAGERS as u32;
    assert!(
        elapsed < serial / 2,
        "asking {MANAGERS} managers took {elapsed:?}, which is about the {serial:?} a loop would \
         take — so they were asked one at a time. Overlapped, this costs about {COST:?}.\n\
         This is #107: `shall sbom` and `shall export` measured 1.1x over 6 waves on a \
         seven-manager host, and no host with four managers could reproduce it until this test."
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
    let registry = registry_of(MANAGERS);
    let state = Arc::new(tokio::sync::Mutex::new(StateRegistry::default()));
    {
        let mut guard = state.lock().await;
        guard.set_managed(
            (0..MANAGERS).map(|i| managed(&format!("mock{i}"), &format!("mock{i}-pkg"))),
        );
    }

    let started = Instant::now();
    let pkgs = shall::app::export::managed_pkgs(&state, &registry, 1).await;
    let elapsed = started.elapsed();

    assert_eq!(pkgs.len(), MANAGERS, "same seven packages");
    assert!(
        elapsed >= COST * (MANAGERS as u32 - 1),
        "a width of one took {elapsed:?}, which is not {MANAGERS} delays of {COST:?} — so the \
         instrument cannot tell a loop from a fan-out, and the test above measures nothing"
    );
}
