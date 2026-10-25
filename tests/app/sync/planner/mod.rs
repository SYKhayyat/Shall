pub use petgraph::algo::{is_cyclic_directed, tarjan_scc};
use petgraph::graph::NodeIndex;
use petgraph::stable_graph::StableDiGraph;
pub use semver::{Version, VersionReq};
use serde::Serialize;
pub use shall::app::sync::planner::*;
pub use shall::backends::BackendRegistry;
pub use shall::config::grammar::Origin;
pub use shall::config::Config;
pub use shall::core::state::ManagedPackage;
pub use shall::core::{Error, GraphAction, PackageSpec, Result, StateRegistry};
pub use shall::model::cycle::{self, Hop};
pub use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
pub use tracing::{debug, info, instrument, warn};
pub use version_compare::{compare as loose_compare, Cmp};

fn managed(name: &str, backend: &str) -> ManagedPackage {
    ManagedPackage {
        name: name.into(),
        backend: backend.into(),
        version: None,
        installed_at: 0,
        expires_at: None,
        options: Default::default(),
        source: "test".into(),
        is_transient: false,
        session_id: None,
    }
}
fn absent_spec(name: &str, backend: &str) -> PackageSpec {
    PackageSpec {
        name: name.into(),
        backend: backend.into(),
        present: false,
        ..PackageSpec::default()
    }
}
struct FakeInstalled {
    name: String,
    installed: Vec<String>,
    /// Its own, so one fake's answer never reaches another's assertions.
    listings: shall::core::installed::InstalledListings,
    /// How many times the manager itself was actually run.
    fetches: Arc<std::sync::atomic::AtomicUsize>,
}
impl shall::core::manager::BackendCore for FakeInstalled {
    fn name(&self) -> &str {
        &self.name
    }
    fn is_available(&self) -> bool {
        true
    }
    fn probes(&self) -> Vec<String> {
        Vec::new()
    }
    fn needs_root(&self) -> bool {
        false
    }
}
#[async_trait::async_trait]
impl shall::core::manager::Queryable for FakeInstalled {
    fn installed_cache(&self) -> (&shall::core::installed::InstalledListings, &str) {
        (&self.listings, &self.name)
    }
    async fn fetch_installed(&self) -> Result<Vec<shall::core::Package>> {
        self.fetches
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(self
            .installed
            .iter()
            .map(|n| shall::core::Package {
                name: n.clone(),
                backend: self.name.clone(),
                version: None,
                properties: HashMap::new(),
            })
            .collect())
    }
    async fn list_manual(&self) -> Result<Vec<shall::core::Package>> {
        self.list_installed().await
    }
    async fn info(&self, name: &str) -> Result<Option<shall::core::Package>> {
        Ok(self
            .list_installed()
            .await?
            .into_iter()
            .find(|p| p.name == name))
    }
}
fn register_fake(
    registry: &mut BackendRegistry,
    backend: &str,
    installed: &[&str],
) -> Arc<std::sync::atomic::AtomicUsize> {
    let fetches = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let fake = Arc::new(FakeInstalled {
        name: backend.to_string(),
        installed: installed.iter().map(|s| s.to_string()).collect(),
        listings: shall::core::installed::InstalledListings::new(),
        fetches: fetches.clone(),
    });
    registry.register(Arc::new(
        shall::core::manager::BackendCapabilities::builder(fake.clone())
            .with_queryable(fake)
            .build(),
    ));
    fetches
}
fn registry_reporting(backend: &str, installed: &[&str]) -> Arc<BackendRegistry> {
    let mut registry = BackendRegistry::new();
    register_fake(&mut registry, backend, installed);
    Arc::new(registry)
}
fn registry_that_cannot_answer(backend: &str) -> Arc<BackendRegistry> {
    struct Mute(String);
    #[async_trait::async_trait]
    impl shall::core::manager::BackendCore for Mute {
        fn name(&self) -> &str {
            &self.0
        }
        fn is_available(&self) -> bool {
            true
        }
        fn probes(&self) -> Vec<String> {
            Vec::new()
        }
        fn needs_root(&self) -> bool {
            false
        }
    }
    let mut registry = BackendRegistry::new();
    registry.register(Arc::new(
        shall::core::manager::BackendCapabilities::builder(Arc::new(Mute(backend.to_string())))
            .build(),
    ));
    Arc::new(registry)
}
struct Unreadable {
    backend: String,
    fails: usize,
    asks: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    listings: shall::core::installed::InstalledListings,
}
impl Unreadable {
    fn new(backend: &str, fails: usize) -> Arc<Self> {
        Arc::new(Self {
            backend: backend.to_string(),
            fails,
            asks: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            listings: shall::core::installed::InstalledListings::new(),
        })
    }
    fn asks(&self) -> usize {
        self.asks.load(std::sync::atomic::Ordering::SeqCst)
    }
    fn core(&self) -> Arc<dyn shall::core::manager::BackendCore> {
        struct Core(String);
        impl shall::core::manager::BackendCore for Core {
            fn name(&self) -> &str {
                &self.0
            }
            fn is_available(&self) -> bool {
                true
            }
            fn probes(&self) -> Vec<String> {
                Vec::new()
            }
            fn needs_root(&self) -> bool {
                false
            }
        }
        Arc::new(Core(self.backend.clone()))
    }
}
#[async_trait::async_trait]
impl shall::core::manager::Queryable for Unreadable {
    fn installed_cache(&self) -> (&shall::core::installed::InstalledListings, &str) {
        // Its own memo, as `FakeInstalled` says in its own field: two fakes sharing one would
        // answer each other's listing.
        (&self.listings, &self.backend)
    }
    async fn fetch_installed(&self) -> shall::core::Result<Vec<shall::core::Package>> {
        Err(shall::core::Error::Other(format!(
            "{} refused to list what it has",
            self.backend
        )))
    }
    async fn list_manual(&self) -> shall::core::Result<Vec<shall::core::Package>> {
        self.fetch_installed().await
    }
    async fn info(&self, _name: &str) -> shall::core::Result<Option<shall::core::Package>> {
        let asked = self.asks.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if asked < self.fails {
            return Err(shall::core::Error::Other(format!(
                "`brew` could not read its cellar: the daemon is not running ({asked})"
            )));
        }
        Ok(None)
    }
}
fn registry_with_unreadable(
    unreadable: &Arc<Unreadable>,
    installed: &[&str],
) -> Arc<BackendRegistry> {
    let mut registry = BackendRegistry::new();
    register_fake(&mut registry, "generic-test", installed);
    registry.register(Arc::new(
        shall::core::manager::BackendCapabilities::builder(unreadable.core())
            .with_queryable(unreadable.clone())
            .build(),
    ));
    Arc::new(registry)
}
async fn plan_with(
    registry: Arc<BackendRegistry>,
    config: &Config,
    hosts: &[&str],
    wanted: &[(&str, &str)],
) -> SyncChanges {
    let state = StateRegistry::new(PathBuf::from("test-state.json"));
    let desired: HashMap<String, Vec<PackageSpec>> = wanted
        .iter()
        .map(|(backend, name)| {
            (
                backend.to_string(),
                vec![PackageSpec {
                    name: (*name).into(),
                    backend: (*backend).into(),
                    ..PackageSpec::default()
                }],
            )
        })
        .collect();
    ChangePlanner::new(registry, &state, config)
        .plan(
            &desired,
            PlanScope::Whole(HostBackends::from_priority(
                hosts.iter().map(|h| h.to_string()).collect(),
            )),
        )
        .await
        .expect("a manager that cannot answer must not fail the plan")
}
async fn absent_removals(registry: Arc<BackendRegistry>, name: &str) -> usize {
    let config = Config::default();
    let state = StateRegistry::new(PathBuf::from("test-state.json"));
    let desired: HashMap<String, Vec<PackageSpec>> = [(
        "generic-test".to_string(),
        vec![absent_spec(name, "generic-test")],
    )]
    .into_iter()
    .collect();
    ChangePlanner::new(registry, &state, &config)
        .plan(&desired, PlanScope::Whole(HostBackends::default()))
        .await
        .unwrap()
        .total_remove()
}
struct DepAnswering {
    deps: HashMap<String, Vec<String>>,
    asked: Arc<std::sync::atomic::AtomicUsize>,
    listings: shall::core::installed::InstalledListings,
}
impl DepAnswering {
    fn new(deps: &[(&str, &[&str])]) -> Arc<Self> {
        Arc::new(Self {
            deps: deps
                .iter()
                .map(|(k, v)| {
                    (
                        (*k).to_string(),
                        v.iter().map(|s| (*s).to_string()).collect(),
                    )
                })
                .collect(),
            asked: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            listings: shall::core::installed::InstalledListings::new(),
        })
    }
}
impl shall::core::manager::BackendCore for DepAnswering {
    fn name(&self) -> &str {
        "deptest"
    }
    fn is_available(&self) -> bool {
        true
    }
    fn probes(&self) -> Vec<String> {
        Vec::new()
    }
    fn needs_root(&self) -> bool {
        false
    }
}
#[async_trait::async_trait]
impl shall::core::manager::Queryable for DepAnswering {
    fn installed_cache(&self) -> (&shall::core::installed::InstalledListings, &str) {
        (&self.listings, "deptest")
    }
    async fn fetch_installed(&self) -> Result<Vec<shall::core::Package>> {
        Ok(Vec::new())
    }
    async fn list_manual(&self) -> Result<Vec<shall::core::Package>> {
        Ok(Vec::new())
    }
    async fn info(&self, _name: &str) -> Result<Option<shall::core::Package>> {
        Ok(None)
    }
}
#[async_trait::async_trait]
impl shall::core::manager::MetadataProvider for DepAnswering {
    async fn get_dependencies(&self, name: &str) -> Result<Vec<String>> {
        self.asked.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(self.deps.get(name).cloned().unwrap_or_default())
    }
}
fn dep_registry(backend: Arc<DepAnswering>) -> Arc<BackendRegistry> {
    let mut registry = BackendRegistry::new();
    registry.register(Arc::new(
        shall::core::manager::BackendCapabilities::builder(backend.clone())
            .with_queryable(backend.clone())
            .with_metadata_provider(backend)
            .build(),
    ));
    Arc::new(registry)
}
fn declared(name: &str) -> PackageSpec {
    let mut options = shall::config::grammar::Options::default();
    options.set("__source", "modules/dev.txt:1".to_string());
    PackageSpec {
        name: name.into(),
        backend: "deptest".into(),
        options,
        requires: vec![],
        present: true,
    }
}
mod a_backend_that_cannot_be_queried_still_plans_the_removal;
mod a_backend_this_machine_does_not_have_plans_no_removal;
mod a_declaration_is_the_only_thing_that_becomes_an_install;
mod a_duplicate_spec_is_won_by_the_first_line_in_its_own_backends_vec;
mod a_manager_that_cannot_answer_costs_only_its_own_declarations;
mod a_manager_that_cannot_answer_schedules_no_install;
mod a_native_dependency_between_declared_packages_does_not_split_the_command;
mod a_package_requiring_itself_is_named;
mod a_plan_runs_each_manager_it_consults_once_and_the_others_never;
mod a_requires_cycle_names_the_packages_and_where_they_came_from;
mod a_scoped_run_does_not_act_on_absent_declarations;
mod a_written_requires_is_still_an_edge;
mod an_absent_declaration_for_something_installed_is_still_a_removal;
mod an_absent_declaration_for_something_not_installed_is_not_a_removal;
mod an_absent_declaration_is_never_installed;
mod an_absent_declaration_is_scheduled_for_removal_even_if_unmanaged;
mod an_imperative_install_is_ordinary_drift_once_nothing_declares_it;
mod an_unreadable_manager_still_plans_its_removals;
mod scope_match_is_exact_entry;
mod scoped_plan_is_non_destructive;
mod sync_never_removes_what_it_does_not_manage;
mod sync_removes_what_it_manages_and_you_stopped_declaring;
mod the_config_says_how_many_times_to_ask_again;
mod the_fake_manager_really_does_answer_a_dependency_query;
mod the_report_is_sorted_for_a_stable_plan;
