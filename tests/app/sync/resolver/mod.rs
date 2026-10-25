use dashmap::DashMap;
pub use semver::{Version, VersionReq};
pub use shall::app::sync::pins::UnmeetablePin;
pub use shall::app::sync::resolver::*;
pub use shall::app::vocab::Vocab;
pub use shall::backends::generic::{
    GenericBackendCore, GenericSearchable, ManagerConfig, ManualListing, SearchSource,
};
pub use shall::backends::BackendRegistry;
pub use shall::config::grammar::{statement, Candidates, Gates, GrammarError, Origin, Statement};
pub use shall::config::parser::HostFacts;
pub use shall::config::Config;
pub use shall::core::executor::{DryRunOutput, MockExecutor};
pub use shall::core::{
    BackendCapabilities, CommandExecutor, Error, LockFile, Package, PackageSpec, Result, Validator,
};
pub use shall::model::resolve::{to_spec, BareAnswer, Provenance, BARE};
pub use shall::model::{DesiredState, Layout, Priority};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Output as StdOutput;
use std::sync::Arc;
pub use tempfile::{tempdir, TempDir};
use tokio::fs;
pub use tracing::{debug, instrument, warn};
pub use version_compare::{compare as loose_compare, Cmp};

struct Repo {
    _tmp: TempDir,
    config: Config,
}
fn repo(files: &[(&str, &str)]) -> Repo {
    let tmp = tempdir().unwrap();
    let root = tmp.path().join("cfg");
    for (path, body) in files {
        let full = root.join(path);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(full, body).unwrap();
    }
    let config = Config {
        // `root` is the repo root the layout hangs off (modules/, profiles/, active).
        config_root: root.clone(),
        ..Config::default()
    };
    Repo { _tmp: tmp, config }
}
async fn resolve(r: &Repo) -> Result<HashMap<String, Vec<PackageSpec>>> {
    let registry = Arc::new(BackendRegistry::new());
    StateResolver::new(&r.config, registry, false)
        .await
        .resolve_desired_state()
        .await
}
fn names(map: &HashMap<String, Vec<PackageSpec>>, backend: &str) -> Vec<String> {
    let mut v: Vec<String> = map
        .get(backend)
        .map(|specs| {
            specs
                .iter()
                .filter(|s| s.present)
                .map(|s| s.name.clone())
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}
fn one_per_line(output: &str) -> shall::parsers::ParseResult {
    shall::parsers::parse_bare_names(output, "test")
}
fn one_per_line_search(output: &str) -> Vec<Package> {
    shall::parsers::parse_bare_names(output, "test").unwrap_or_default()
}
fn manager(name: &str, exec: CommandExecutor) -> Arc<BackendCapabilities> {
    let config = ManagerConfig {
        name: name.into(),
        binary: None,
        remove_binary: None,
        install_args: vec![],
        remove_args: vec![],
        list_args: vec![],
        manual: ManualListing::AllInstalled,
        essential_args: None,
        search_args: vec!["search".into()],
        search_binary: None,
        enumerate_args: None,
        enumerate_binary: None,
        list_binary: None,
        upgrade_args: vec![],
        update_args: None,
        purge_args: None,
        orphan_dry_run: None,
        foreign_args: None,
        repo_add_args: None,
        repo_remove_args: None,
        depends: None,
        clean_cache: None,
        repo_list_args: None,
        repo_binary: None,
        repo_list_binary: None,
        repo_remove_binary: None,
        repo_list_shape: shall::backends::generic::RepoListing::Columns,
        version_pin: None,
        needs_root: false,
        is_exclusive: false,
        install_source_option: None,
        extra_probes: None,
        upgrade_reinstall_args: None,
        property_probes: Vec::new(),
        machine_list: None,
        outdated: None,
        search_source: SearchSource::Command,
        qualified_names: false,
    };
    let core = Arc::new(GenericBackendCore {
        name: name.into(),
        executor: exec,
        config,
        parser: Arc::new(shall::parsers::LambdaParser {
            installed_fn: one_per_line,
            search_fn: one_per_line_search,
        }),
    });
    Arc::new(
        BackendCapabilities::builder(core.clone())
            .with_searchable(Arc::new(GenericSearchable { core }))
            .build(),
    )
}
fn registry(first: StdOutput, second: StdOutput) -> Arc<BackendRegistry> {
    let vfs: Arc<DashMap<PathBuf, String>> = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    mock.set_command_exists("first", true);
    mock.set_command_exists("second", true);
    mock.set_response("first search jq", Ok(first));
    mock.set_response("second search jq", Ok(second));
    let exec = CommandExecutor::with_layer(false, false, mock, vfs, Arc::new(DashMap::new()));
    let mut reg = BackendRegistry::new();
    reg.register(manager("first", exec.clone()));
    reg.register(manager("second", exec));
    Arc::new(reg)
}
fn found() -> StdOutput {
    let mut out: StdOutput = DryRunOutput::new().into();
    out.stdout = b"jq\n".to_vec();
    out
}
async fn settle(r: &Repo, reg: Arc<BackendRegistry>) -> Result<(String, Option<String>)> {
    let state = StateResolver::new(&r.config, reg, false)
        .await
        .recording_locks()
        .resolve_model()
        .await?;
    let backend = state
        .packages
        .iter()
        .find(|(_, specs)| specs.iter().any(|s| s.name == "jq"))
        .map(|(b, _)| b.clone())
        .expect("jq resolved somewhere");
    let lock = shall::core::BareLock::load(&shall::core::BareLock::path_in(
        &r.config.layout().locks_dir(),
    ))
    .unwrap();
    let recorded = lock.get("jq").map(str::to_string);
    Ok((backend, recorded))
}
fn bare_jq() -> Repo {
    repo(&[
        ("priority", "first\nsecond\n"),
        ("active", "Work\n"),
        ("profiles/Work", "use base\n"),
        ("modules/base.txt", "jq\n"),
    ])
}
mod a_backend_missing_from_priority_is_refused_by_name;
mod a_constraint_nothing_can_read_is_not_treated_as_met;
mod a_contradiction_across_two_modules_is_an_error_naming_both;
mod a_manager_that_could_not_answer_leaves_no_lock;
mod a_manager_that_said_no_lets_the_pick_be_recorded;
mod a_missing_priority_file_is_an_error_that_names_it;
mod a_module_reached_through_another_module_keeps_its_own_scope;
mod a_package_is_scoped_to_its_module_and_to_the_profile_that_reaches_it;
mod a_pin_is_satisfied_by_semver_then_by_the_literal_then_loosely;
mod an_absent_line_crosses_the_seam_marked_absent;
mod an_unreached_broken_module_is_never_parsed;
mod nothing_found_past_a_silent_manager_says_so;
mod the_chain_answers_in_the_chains_order_however_the_answers_arrive;
mod the_seam_carries_what_the_active_profiles_reach;
