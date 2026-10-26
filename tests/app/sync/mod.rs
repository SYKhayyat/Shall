//! The tests that mirror `src/app/sync/`, module for module.
//!
//! **The mirror is the rule.** `src/app/sync/mod.rs` owns `tests/app/sync/`, and every test
//! for the sync engine lives in a file of its own, named after the assertion it makes. The
//! helpers the assertions share stay here, in the module file, because a helper in `src/` could
//! not be reached from a test crate — `tests/` links the library without `cfg(test)`.
//!
//! **The engine's own tests were in the top level of `tests/` until now**, in files whose
//! subject was the engine and whose name was an assertion; they are here because the engine
//! has a home and a home is where its tests go.

mod a_declared_health_check_with_nothing_to_revert_it_stops_the_sync;
mod a_deliberate_purge_is_not_refused_by_the_accidental_removal_ceiling;
mod a_dry_run_does_not_rebuild_the_out_of_tree_modules;
mod a_failed_snapshot_prune_is_reported_instead_of_being_swallowed;
mod a_health_command_that_was_never_approved_stops_the_sync;
mod a_kernel_change_rebuilds_the_out_of_tree_modules;
mod a_module_that_did_not_build_fails_the_sync_loudly;
mod a_partial_sync_counts_only_the_operations_that_happened;
mod a_rebuild_is_narrated_as_a_rebuild;
mod a_successful_sync_prunes_the_snapshots_the_retention_policy_doomed;
mod an_ordinary_sync_is_refused_by_the_accidental_removal_ceiling;
mod guard;
mod pin_advice;
mod pins;
mod planner;
mod recovery_carries_past_a_failure_without_rolling_back_the_success;
mod recovery_uses_its_own_transaction_policy;
mod resolver;
mod saved_plan;
mod the_advice_names_the_thing_to_do_about_it;
mod the_declared_health_checks_are_the_declared_ones;
mod the_engine_finds_the_hooks_this_machine_wrote;
mod the_probe_answers_both_ways;

use dashmap::DashMap;
use shall::app::sync::guard::GuardScope;
use shall::app::sync::planner::SyncChanges;
use shall::app::sync::SyncEngine;
use shall::config::Config;
use shall::core::executor::{CommandExecutor, MockExecutor};
use shall::core::PackageSpec;
use std::fmt::Write as _;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing::Subscriber;

/// A config directory of its own, so a test's writes land nowhere else.
pub(crate) fn config_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("shall-sync-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("the config directory");
    dir
}

/// The engine, wired to a mock layer, with the config the test asked for.
pub(crate) async fn engine(config: Config) -> (SyncEngine, Arc<MockExecutor>) {
    let vfs: Arc<DashMap<PathBuf, String>> = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    // **No snapshot provider on this machine.** The built-in providers are *detected* — a row
    // whose `detect` program is missing is not a provider — and the mock answers `true` to
    // every existence probe unless it is told otherwise. So they are told otherwise here: a
    // machine that could revert a change is a different machine, and the tests that are about
    // the "could not revert" half need this one.
    for program in ["btrfs", "timeshift", "tmutil", "powershell", "zfs"] {
        mock.command_existence.insert(program.to_string(), false);
    }
    let exec =
        CommandExecutor::with_layer(true, false, mock.clone(), vfs, Arc::new(DashMap::new()));
    // **The state file goes beside the config the test handed over**, and the directory is not
    // wiped here: a test that wrote a hook or a preferences file into it means it.
    let state = config.config_root.join("state.json");
    let app = shall::app::App::new_with_executor_and_state_path(config, exec, Some(state))
        .await
        .expect("the app");
    (SyncEngine::new(app.machinery()), mock)
}

/// A config that declares the health checks the test wrote, and nothing else.
pub(crate) fn config_with_health(tag: &str, health: &[&str]) -> Config {
    let dir = config_dir(tag);
    std::fs::write(dir.join("preferences.toml"), "").expect("the preferences");
    let mut config = Config::sandboxed(&dir);
    config.health = health.iter().map(|h| h.to_string()).collect();
    config
}

/// Ask the engine's own probe, which is what decides whether a change is rolled back.
pub(crate) async fn engine_probe_ok(probe: &shall::model::health::Probe) -> bool {
    SyncEngine::probe_ok(probe).await
}

/// A plan that changes nothing, so the checks under test are the only thing in play.
pub(crate) fn no_changes() -> SyncChanges {
    SyncChanges {
        graph: petgraph::stable_graph::StableDiGraph::new(),
        install_map: std::collections::HashMap::new(),
        removal_tracker: std::collections::HashSet::new(),
        skipped: Vec::new(),
    }
}

pub(crate) fn package(backend: &str, name: &str) -> PackageSpec {
    PackageSpec {
        backend: backend.to_string(),
        name: name.to_string(),
        options: Default::default(),
        requires: Vec::new(),
        present: false,
    }
}

pub(crate) fn install_changes(packages: &[PackageSpec]) -> SyncChanges {
    let mut changes = SyncChanges::default();
    changes.add_installs(packages);
    changes
}

pub(crate) fn removal_changes(packages: &[(&str, &str)]) -> SyncChanges {
    let mut changes = SyncChanges::default();
    for (backend, name) in packages {
        changes.add_removal(backend, name);
    }
    changes
}

pub(crate) fn log_collector() -> (Arc<Mutex<Vec<String>>>, tracing::subscriber::DefaultGuard) {
    struct Collect(Arc<Mutex<Vec<String>>>);

    impl Subscriber for Collect {
        fn enabled(&self, _: &tracing::Metadata<'_>) -> bool {
            true
        }

        fn new_span(&self, _: &tracing::span::Attributes<'_>) -> tracing::span::Id {
            tracing::span::Id::from_u64(1)
        }

        fn record(&self, _: &tracing::span::Id, _: &tracing::span::Record<'_>) {}

        fn record_follows_from(&self, _: &tracing::span::Id, _: &tracing::span::Id) {}

        fn event(&self, event: &tracing::Event<'_>) {
            let mut visitor = MessageVisitor(Arc::new(Mutex::new(String::new())));
            event.record(&mut visitor);
            let text = std::mem::take(&mut *visitor.0.lock().expect("the log mutex"));
            self.0.lock().expect("the log mutex").push(text);
        }

        fn enter(&self, _: &tracing::span::Id) {}

        fn exit(&self, _: &tracing::span::Id) {}
    }

    struct MessageVisitor(Arc<Mutex<String>>);

    impl tracing::field::Visit for MessageVisitor {
        fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
            let _ = write!(self.0.lock().expect("the log mutex"), "{} ", field.name());
            let _ = write!(self.0.lock().expect("the log mutex"), "{value:?}");
        }
    }

    let seen = Arc::new(Mutex::new(Vec::new()));
    let guard = tracing::subscriber::set_default(Collect(seen.clone()));
    (seen, guard)
}

mod a_machine_converges_tests;
mod critical_paths_tests;
