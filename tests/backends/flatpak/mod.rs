use async_trait::async_trait;
pub use shall::backends::flatpak::*;
pub use shall::core::{
    BackendCore, CommandExecutor, Installable, MetadataProvider, Package, PackageSpec, Queryable,
    Result, Searchable, Upgradable,
};
pub use shall::model::scope::Scope;
pub use shall::parsers::{or_unrecognised, ParseResult};
pub use shall::utils::text::sanitize;
use std::collections::HashMap;
use std::sync::Arc;
pub use tracing::{debug, info};

fn spec_with(name: &str, options: &[(&str, &str)]) -> PackageSpec {
    PackageSpec {
        name: name.to_string(),
        backend: "flatpak".to_string(),
        options: options
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
        ..Default::default()
    }
}
fn settings(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}
fn scripted_without_a_listing() -> (
    Arc<FlatpakBackendCore>,
    Arc<shall::core::executor::MockExecutor>,
) {
    let vfs = Arc::new(dashmap::DashMap::new());
    let mock = Arc::new(shall::core::executor::MockExecutor::new(vfs.clone()));
    let exec = CommandExecutor::with_layer(
        false,
        false,
        mock.clone(),
        vfs,
        Arc::new(dashmap::DashMap::new()),
    );
    (
        Arc::new(FlatpakBackendCore::new(exec, FLATPAK_DEFAULT_SCOPE)),
        mock,
    )
}
fn scripted(
    listing: &str,
) -> (
    Arc<FlatpakBackendCore>,
    Arc<shall::core::executor::MockExecutor>,
) {
    let vfs = Arc::new(dashmap::DashMap::new());
    let mock = Arc::new(shall::core::executor::MockExecutor::new(vfs.clone()));
    mock.set_response(
        "flatpak list --app --columns=application,version,branch",
        Ok(shall::core::executor::DryRunOutput {
            stdout: listing.as_bytes().to_vec(),
            ..Default::default()
        }
        .into()),
    );
    let exec = CommandExecutor::with_layer(
        false,
        false,
        mock.clone(),
        vfs,
        Arc::new(dashmap::DashMap::new()),
    );
    (
        Arc::new(FlatpakBackendCore::new(exec, FLATPAK_DEFAULT_SCOPE)),
        mock,
    )
}
mod a_branch_switch_names_the_app_and_the_branch_behind_the_terminator;
mod a_first_install_on_a_declared_branch_is_not_a_switch;
mod a_row_with_no_trailing_columns_is_still_a_package;
mod a_scope_that_is_neither_word_is_refused;
mod an_already_installed_ref_is_not_an_error_to_shall;
mod an_app_on_two_branches_reports_no_branch_at_all;
mod an_empty_version_column_does_not_slide_the_branch_into_it;
mod an_install_that_declares_no_channel_never_asks_for_the_listing;
mod an_unreadable_or_matching_branch_issues_no_switch;
mod every_way_of_writing_the_scope_reaches_the_argv;
mod flatpak_channel_becomes_the_branch_of_the_ref;
mod flatpak_refs_come_after_the_terminator;
mod flatpak_search_prefers_app_id;
mod flatpak_without_a_channel_installs_the_bare_name;
mod flatpaks_other_name_carrying_commands_terminate_too;
mod switching_branch_installs_the_ref_and_then_points_the_app_at_it;
mod the_boolean_this_key_used_to_be_is_refused_and_names_its_replacement;
