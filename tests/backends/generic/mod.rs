pub use shall::backends::generic::verification_note;
use shall::core::{
    CommandExecutor, Installable, MetadataProvider, Package, PackageSpec, Queryable, RepoManager,
    Upgradable,
};
use std::sync::Arc;
const REFUSAL: &str = "Error: plugin source does not support verification";
use dashmap::DashMap;
pub use shall::backends::generic::*;
pub use shall::core::executor::{DryRunOutput, MockExecutor};
pub use shall::parsers::LambdaParser;
fn apt_like_core(
    mock: Arc<MockExecutor>,
    vfs: Arc<DashMap<std::path::PathBuf, String>>,
) -> GenericBackendCore {
    let exec = CommandExecutor::with_layer(true, false, mock, vfs, Arc::new(DashMap::new()));
    apt_like_core_named("apt", exec)
}
fn apt_like_core_named(name: &str, exec: CommandExecutor) -> GenericBackendCore {
    GenericBackendCore {
        name: name.into(),
        executor: exec,
        config: ManagerConfig {
            name: name.into(),
            binary: None,
            remove_binary: None,
            install_args: vec![],
            remove_args: vec![],
            list_args: vec![],
            manual: ManualListing::AllInstalled,
            essential_args: None,
            search_args: vec![],
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
            repo_list_args: None,
            repo_binary: None,
            repo_list_binary: None,
            repo_remove_binary: None,
            repo_list_shape: RepoListing::Columns,
            depends: Some(DependsProbe {
                binary: None,
                args: vec![
                    "depends".into(),
                    "--no-recommends".into(),
                    "--no-suggests".into(),
                    "{name}".into(),
                ],
                parse: Arc::new(parse_dependency_output),
            }),
            clean_cache: None,
            version_pin: None,
            needs_root: true, // apt needs root for writes — but reads must NOT escalate
            is_exclusive: true,
            install_source_option: None,
            extra_probes: None,
            upgrade_reinstall_args: None,
            property_probes: Vec::new(),
            machine_list: None,
            outdated: None,
            search_source: SearchSource::Command,
            qualified_names: false,
        },
        parser: Arc::new(LambdaParser {
            installed_fn: |_| Ok(vec![]),
            search_fn: |_| vec![],
        }),
    }
}
fn queryable_with(
    manual: ManualListing,
    mock: Arc<MockExecutor>,
    vfs: Arc<DashMap<std::path::PathBuf, String>>,
) -> GenericQueryable {
    let mut core = apt_like_core(mock, vfs);
    core.config.list_binary = Some("dpkg-query".into());
    core.config.list_args = vec![
        "-W".into(),
        "-f=${db:Status-Status} ${Package} ${Version}\\n".into(),
    ];
    core.config.manual = manual;
    core.parser = Arc::new(shall::parsers::apt::AptParser);
    GenericQueryable {
        core: Arc::new(core),
    }
}
fn source_option_core(
    mock: Arc<MockExecutor>,
    vfs: Arc<DashMap<std::path::PathBuf, String>>,
) -> GenericBackendCore {
    let mut core = apt_like_core(mock, vfs);
    core.name = "helm".into();
    core.config.name = "helm".into();
    core.config.install_source_option = Some("url".into());
    core.config.install_args = vec!["plugin".into(), "install".into()];
    core.config.remove_args = vec!["plugin".into(), "uninstall".into()];
    core.config.needs_root = false;
    core
}
fn spec_with(name: &str, opts: &[(&str, &str)]) -> PackageSpec {
    PackageSpec {
        name: name.into(),
        backend: "helm".into(),
        options: opts
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
        ..Default::default()
    }
}
fn pip_core(
    mock: Arc<MockExecutor>,
    vfs: Arc<DashMap<std::path::PathBuf, String>>,
) -> GenericBackendCore {
    let exec = CommandExecutor::with_layer(true, false, mock, vfs, Arc::new(DashMap::new()));
    let mut core = apt_like_core_named("pip", exec);
    core.config.install_args = vec!["install".into()];
    core
}
fn pip_spec(name: &str, system: bool) -> PackageSpec {
    PackageSpec {
        name: name.into(),
        backend: "pip".into(),
        options: if system {
            [("system".to_string(), "true".to_string())]
                .into_iter()
                .collect()
        } else {
            Default::default()
        },
        ..Default::default()
    }
}
fn this_hosts_helm_takes_the_opt_out() -> bool {
    shall::core::tool_help::accepts_flag(
        "helm",
        &["plugin".to_string(), "install".to_string()],
        "--verify=false",
    ) != Some(false)
}
fn repo_mgr(
    name: &str,
    mock: Arc<MockExecutor>,
    vfs: Arc<DashMap<std::path::PathBuf, String>>,
    edit: impl FnOnce(&mut ManagerConfig),
) -> GenericRepoManager {
    let mut core = apt_like_core(mock, vfs);
    core.name = name.to_string();
    core.config.name = name.to_string();
    edit(&mut core.config);
    GenericRepoManager {
        core: Arc::new(core),
    }
}
fn apk_repo(
    mock: Arc<MockExecutor>,
    vfs: Arc<DashMap<std::path::PathBuf, String>>,
) -> GenericRepoManager {
    repo_mgr("apk", mock, vfs, |c| {
        c.repo_add_args = Some(vec![
            "-c".into(),
            "echo '{url}' >> /etc/apk/repositories".into(),
        ]);
        c.repo_remove_args = Some(vec![
            "-c".into(),
            "sed -i '\\|{url}|d' /etc/apk/repositories".into(),
        ]);
        c.repo_list_args = Some(vec!["/etc/apk/repositories".into()]);
        c.repo_binary = Some("sh".into());
        c.repo_list_binary = Some("cat".into());
    })
}
use std::collections::HashMap;
fn settings(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}
fn row(name: &str) -> ManagerConfig {
    ManagerConfig {
        name: name.into(),
        binary: None,
        remove_binary: None,
        install_args: vec![],
        remove_args: vec![],
        purge_args: None,
        list_args: vec![],
        manual: ManualListing::Unsupported,
        essential_args: None,
        search_args: vec![],
        search_binary: None,
        enumerate_args: None,
        enumerate_binary: None,
        list_binary: None,
        upgrade_args: vec![],
        update_args: None,
        orphan_dry_run: None,
        foreign_args: None,
        repo_add_args: None,
        repo_remove_args: None,
        repo_list_args: None,
        repo_binary: None,
        repo_list_binary: None,
        repo_remove_binary: None,
        repo_list_shape: RepoListing::Columns,
        depends: None,
        clean_cache: None,
        version_pin: None,
        install_source_option: None,
        needs_root: false,
        is_exclusive: false,
        property_probes: Vec::new(),
        machine_list: None,
        outdated: None,
        search_source: SearchSource::Command,
        qualified_names: false,
        upgrade_reinstall_args: None,
        extra_probes: None,
    }
}

mod a_line_that_did_not_ask_keeps_verification_on;
mod a_listed_name_resolves_to_its_url;
mod a_lister_that_died_silently_is_a_failure_not_an_empty_machine;
mod a_lister_that_succeeded_with_nothing_to_say_still_reports_an_empty_machine;
mod a_manager_that_answers_the_machine_format_is_not_asked_twice;
mod a_manager_that_demands_a_version_is_given_one;
mod a_manager_that_refuses_the_machine_format_is_read_from_its_text_listing;
mod a_mixed_batch_does_not_hand_one_lines_opt_out_to_another;
mod a_placeholder_is_recognised_wherever_it_sits_in_the_argument;
mod a_placeholder_with_nothing_behind_it_refuses_rather_than_emptying;
mod a_repo_named_by_something_that_is_not_a_url_is_refused;
mod a_row_that_names_no_setting_is_unchanged;
mod a_setting_reaches_the_argv_it_is_named_in;
mod a_template_with_an_unfillable_placeholder_is_refused;
mod a_url_that_could_break_the_shell_template_is_refused_before_anything_runs;
mod a_verification_failure_names_the_flag_a_declaration_can_write;
mod add_repo_still_substitutes_name_and_url;
mod advice_never_names_a_flag_the_tool_would_drop;
mod all_installed_backends_still_report_their_installed_set;
mod an_empty_source_option_is_as_missing_as_no_option;
mod an_option_after_the_name_is_recognised_by_its_dash_and_not_by_its_variant;
mod an_unrelated_failure_earns_no_verification_advice;
mod an_upgrade_whose_every_attempt_fails_is_reported;
mod apk_repo_removal_carries_the_real_url_and_no_placeholder;
mod apt_manual_list_asks_apt_mark_not_dpkg_query;
mod backends_that_share_a_package_manager_share_its_lock;
mod batch_bound_respects_both_caps_and_always_takes_one;
mod concrete_version_rejects_floating;
mod essential_query_is_absent_unless_a_backend_declares_it;
mod every_argv_bearing_field_is_visited;
mod get_dependencies_parses_names_without_sudo;
mod install_from_a_source_option_sends_the_source_and_removes_by_name;
mod install_without_the_source_option_refuses_and_names_the_fix;
mod list_orphans_reports_unsupported_without_a_dry_run;
mod pip_without_the_opt_in_sends_no_flag;
mod preamble_and_metadata_are_not_dependencies;
mod remove_reports_unsupported_with_empty_remove_args;
mod substitution_happens_inside_a_token;
mod system_reaches_pip_as_its_own_flag_and_is_never_shared;
mod the_default_is_used_when_the_key_is_absent_or_blank;
mod the_leftover_scan_can_actually_fail;
mod unsupported_backend_reports_nothing_rather_than_everything;
mod unverified_becomes_the_managers_own_opt_out_flag;
mod upgrading_by_reinstall_is_one_command_when_it_works;
mod version_pin_renders_native_syntax;

mod the_probe_argv_does_not_depend_on_the_package_tests;
