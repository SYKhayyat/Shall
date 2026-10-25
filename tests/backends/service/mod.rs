use async_trait::async_trait;
use dashmap::DashMap;
use serde::Deserialize;
pub use shall::backends::service::*;
pub use shall::core::adapter::{self, AdapterRow, Detected};
pub use shall::core::executor::MockExecutor;
pub use shall::core::{
    BackendCore, CommandExecutor, Error, Installable, MetadataProvider, Package, PackageSpec,
    Queryable, Result,
};
use std::borrow::Cow;
use std::process::Output;
use std::sync::Arc;
pub use tracing::{info, warn};

fn test_row() -> InitProvider {
    toml::from_str(
        r#"
        name = "testinit"
        detect = "testinit-detect"
        stop = [["stop-cmd", "{name}"]]
        disable = [["disable-cmd", "{name}"]]
        stop_benign_exits = [42]
        "#,
    )
    .expect("fixture provider row")
}
fn exit_of(code: i32) -> Result<Output> {
    Ok(Output {
        status: shall::core::executor::fabricate_status(code),
        stdout: Vec::new(),
        stderr: b"manager said something".to_vec(),
    })
}
fn service_layer(responses: &[(&str, Result<Output>)]) -> (Arc<MockExecutor>, CommandExecutor) {
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    for (cmd, res) in responses {
        mock.set_response(cmd, res.clone());
    }
    let exec =
        CommandExecutor::with_layer(false, false, mock.clone(), vfs, Arc::new(DashMap::new()));
    (mock, exec)
}
fn core_with(exec: CommandExecutor) -> ServiceInstallable {
    ServiceInstallable {
        core: Arc::new(ServiceBackendCore::with_providers(exec, vec![test_row()])),
    }
}
async fn removal_token() -> shall::app::sync::guard::Reaped {
    crate::harness::reaped_for_a_test(shall::app::sync::guard::GuardScope::Sync).await
}
fn shipped(name: &str) -> InitProvider {
    providers(vec![])
        .into_iter()
        .find(|p| p.name == name)
        .unwrap_or_else(|| panic!("{} must ship", name))
}
mod a_derived_restart_derives_from_the_domain_it_is_in;
mod a_derived_restart_labels_each_half_with_its_own_verb;
mod a_per_user_listing_comes_from_the_per_user_row;
mod a_real_removal_failure_is_reported_not_recorded_removed;
mod a_row_missing_start_or_stop_is_refused;
mod a_row_without_restart_derives_stop_then_start;
mod a_user_row_cannot_redefine_a_builtin;
mod an_init_with_no_per_user_domain_refuses_a_named_account_by_name;
mod every_systemd_row_suppresses_the_pager;
mod launchd_ends_its_options_before_the_service;
mod launchd_listing_skips_its_header;
mod launchd_restart_is_stop_then_start;
mod one_broken_service_does_not_strand_the_rest_of_the_sweep;
mod openrc_uses_rc_update_and_rc_service;
mod options_are_independent;
mod options_default_to_enable_and_start;
mod removal_forgives_the_declared_already_in_state_code;
mod systemd_installed_but_not_booted_loses_to_the_init_that_is_running;
mod systemd_listing_strips_the_service_suffix;
mod systemd_maps_each_action_and_ends_its_options_before_the_unit;
mod the_benign_code_reaches_the_verb_that_declared_it_and_no_other;
mod the_inits_that_answer_zero_declare_no_benign_codes;
mod the_other_inits_deliberately_emit_no_terminator;
mod the_per_user_domain_is_expressed_the_way_each_init_needs;
mod the_shipped_table_parses_and_carries_the_five_inits;
mod the_shipped_table_says_which_inits_have_a_per_user_domain;
mod the_systemd_row_names_the_file_sd_booted_checks;
mod windows_forgives_already_in_that_state_once_per_verb;
mod windows_listing_reads_the_service_name_field;
mod windows_restart_is_stop_then_start;
