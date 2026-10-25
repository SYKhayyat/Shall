use chrono::Local;
use dashmap::DashMap;
pub use shall::app::adopt::*;
pub use shall::backends::generic::{
    GenericBackendCore, GenericQueryable, ManagerConfig, ManualFormat, ManualListing, SearchSource,
};
pub use shall::backends::BackendRegistry;
pub use shall::config::Config;
pub use shall::core::executor::{DryRunOutput, MockExecutor};
pub use shall::core::{BackendCapabilities, CommandExecutor, Package, Result, StateRegistry};
pub use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
pub use tracing::{debug, info, instrument, trace, warn};

fn over(reg: Arc<BackendRegistry>) -> shall::app::Backends {
    let names = reg.all().iter().map(|b| b.name().to_string()).collect();
    shall::app::Backends::new(
        reg,
        Ok(shall::model::priority::Priority::from_backends(names)),
    )
}
fn registry_with(manual: ManualListing, mock: Arc<MockExecutor>) -> Arc<BackendRegistry> {
    let vfs: Arc<DashMap<PathBuf, String>> = Arc::new(DashMap::new());
    mock.set_command_exists("apt", true);
    let exec =
        CommandExecutor::with_layer(true, false, mock.clone(), vfs, Arc::new(DashMap::new()));
    let mut config = ManagerConfig {
        name: "apt".into(),
        binary: None,
        remove_binary: None,
        install_args: vec![],
        remove_args: vec![],
        list_args: vec!["-W".into()],
        manual: ManualListing::AllInstalled,
        essential_args: Some(vec![
            "-W".into(),
            "-f=${Essential} ${Priority} ${Package}\\n".into(),
        ]),
        search_args: vec![],
        search_binary: None,
        enumerate_args: None,
        enumerate_binary: None,
        list_binary: Some("dpkg-query".into()),
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
        search_source: SearchSource::Command,
        qualified_names: false,
    };
    config.manual = manual;
    let core = Arc::new(GenericBackendCore {
        name: "apt".into(),
        executor: exec,
        config,
        // apt's real parser: `LambdaParser` has no `parse_essential`, so it inherits
        // the trait default and answers "nothing is essential" — which would make a
        // test about essential packages pass for the wrong reason.
        parser: Arc::new(shall::parsers::apt::AptParser),
    });
    let mut reg = BackendRegistry::new();
    reg.register(Arc::new(
        BackendCapabilities::builder(core.clone())
            .with_queryable(Arc::new(GenericQueryable { core }))
            .build(),
    ));
    Arc::new(reg)
}
fn adopter(reg: Arc<BackendRegistry>) -> Adopter {
    let config = Config {
        // Keep the default protected list out of these assertions.
        guard: shall::config::GuardSettings {
            protected_packages: vec![],
            ..Default::default()
        },
        ..Config::default()
    };
    let state = Arc::new(Mutex::new(StateRegistry::default()));
    Adopter::new(over(reg), state, &config)
}
fn alpm_client(
    reg: &mut BackendRegistry,
    exec: &CommandExecutor,
    name: &'static str,
    installed_fn: fn(&str) -> shall::parsers::ParseResult,
    search_fn: fn(&str) -> Vec<shall::core::Package>,
) {
    let core = Arc::new(GenericBackendCore {
        name: name.into(),
        executor: exec.clone(),
        config: ManagerConfig {
            list_args: vec!["-Q".into()],
            manual: ManualListing::Command {
                binary: None,
                args: vec!["-Qe".into()],
                format: ManualFormat::SameAsInstalled,
            },
            ..shall::backends::registry::base_config(name)
        },
        parser: Arc::new(shall::parsers::LambdaParser {
            installed_fn,
            search_fn,
        }),
    });
    reg.register(Arc::new(
        BackendCapabilities::builder(core.clone())
            .with_queryable(Arc::new(GenericQueryable { core }))
            .build(),
    ));
}
fn alpm_registry(mock: Arc<MockExecutor>, order: [&'static str; 3]) -> Arc<BackendRegistry> {
    let vfs: Arc<DashMap<PathBuf, String>> = Arc::new(DashMap::new());
    let exec =
        CommandExecutor::with_layer(true, false, mock.clone(), vfs, Arc::new(DashMap::new()));
    let mut reg = BackendRegistry::new();
    for name in order {
        mock.set_command_exists(name, true);
        // The same three lines from all three, because there is one database under them.
        mock.set_response(
            &format!("{} -Qe", name),
            Ok(DryRunOutput {
                stdout: b"bash 5.2.037-1\njq 1.8.2-1\nyay 13.0.1-1\n".to_vec(),
                stderr: vec![],
            }
            .into()),
        );
        match name {
            "pacman" => alpm_client(
                &mut reg,
                &exec,
                "pacman",
                |o| shall::parsers::pacman::parse_list_for(o, "pacman"),
                |o| shall::parsers::pacman::parse_search_for(o, "pacman"),
            ),
            "yay" => alpm_client(
                &mut reg,
                &exec,
                "yay",
                |o| shall::parsers::pacman::parse_list_for(o, "yay"),
                |o| shall::parsers::pacman::parse_search_for(o, "yay"),
            ),
            _ => alpm_client(
                &mut reg,
                &exec,
                "paru",
                |o| shall::parsers::pacman::parse_list_for(o, "paru"),
                |o| shall::parsers::pacman::parse_search_for(o, "paru"),
            ),
        }
    }
    Arc::new(reg)
}
mod a_name_no_line_can_hold_is_reported_rather_than_written;
mod a_name_the_owner_already_manages_is_not_adopted_under_a_client;
mod a_protected_package_is_adopted_like_any_other;
mod a_service_is_declared_as_the_state_it_was_found_in;
mod adopts_nothing_from_a_backend_that_cannot_report_intent;
mod adopts_only_the_packages_the_backend_reports_as_user_chosen;
mod an_os_essential_package_is_a_live_line;
mod an_os_essential_package_is_adopted_like_any_other;
mod audit_and_adopt_cannot_disagree;
mod the_database_owner_claims_the_name_whichever_client_answers_first;
mod the_manifest_warns_that_it_is_an_estimate_and_names_its_source;
mod three_clients_of_one_package_database_adopt_each_package_once;
