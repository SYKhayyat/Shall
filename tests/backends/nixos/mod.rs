use async_trait::async_trait;
use dashmap::DashMap;
pub use shall::backends::nixos::*;
pub use shall::core::executor::MockExecutor;
pub use shall::core::{
    BackendCore, CommandExecutor, Error, Installable, MetadataProvider, Package, PackageSpec,
    Queryable, Result,
};
pub use std::collections::{BTreeMap, BTreeSet};
pub use std::path::{Path, PathBuf};
use std::sync::Arc;
pub use tracing::{debug, info};

fn dry_core() -> (
    NixosBackendCore,
    Arc<DashMap<std::path::PathBuf, String>>,
    Arc<MockExecutor>,
) {
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    let exec = CommandExecutor::with_layer(
        true,
        false,
        mock.clone(),
        vfs.clone(),
        Arc::new(DashMap::new()),
    );
    let core = NixosBackendCore::new(
        exec,
        std::env::temp_dir().join("shall-nixos-test-cfg"),
        true,
    );
    (core, vfs, mock)
}
fn set(items: &[&str]) -> BTreeSet<String> {
    items.iter().map(|s| s.to_string()).collect()
}
fn packages(items: &[&str]) -> Module {
    Module {
        packages: set(items),
        ..Module::default()
    }
}
mod a_dry_run_records_the_generated_file_instead_of_writing_it;
mod a_dry_run_unplace_removes_nothing;
mod a_name_carrying_nix_syntax_is_refused_not_interpolated;
mod a_package_write_carries_the_services_and_ports_it_knows_nothing_about;
mod a_real_unplace_still_removes_and_forgives_already_gone;
mod a_service_declaration_is_split_into_what_nix_says_and_what_the_init_does;
mod a_shape_with_nowhere_to_put_it_is_refused;
mod an_empty_set_still_renders_a_valid_module;
mod an_existing_import_is_recognised_however_it_was_written;
mod an_existing_imports_list_gains_an_entry_rather_than_a_second_list;
mod every_edited_configuration_is_written_for_nix_to_parse;
mod every_rendered_shape_is_written_for_nix_to_parse;
mod module_resolve_secrets_replaces_all_config_text_entries;
mod render_with_secrets_fails_on_missing_secret;
mod render_with_secrets_injects_config_text_values;
mod services_and_ports_are_rendered_where_nixos_reads_them;
mod substitute_secrets_leaves_text_without_secrets_unchanged;
mod substitute_secrets_replaces_secret_references;
mod substitute_secrets_returns_error_for_missing_secret;
mod the_escalated_half_of_a_dry_run_never_stages_a_real_temp_file;
mod the_generated_file_is_a_nixos_module_naming_its_packages;
mod the_import_goes_inside_the_attribute_set_not_after_it;
mod the_reader_leaves_the_package_list_where_it_ends;
mod what_is_rendered_is_what_is_read_back;
