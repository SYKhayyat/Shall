use async_trait::async_trait;
use serde::Deserialize;
pub use shall::backends::setting::*;
pub use shall::core::adapter::{self, AdapterRow, Detected};
pub use shall::core::{
    BackendCore, CommandExecutor, Error, Installable, MetadataProvider, Package, PackageSpec,
    Queryable, Result,
};
pub use shall::model::scope::Scope;
use std::sync::Arc;
pub use tracing::{info, warn};

fn gsettings() -> SettingAdapter {
    adapters(vec![])
        .into_iter()
        .find(|a| a.name == "gsettings")
        .expect("the shipped table must carry gsettings")
}
fn row(name: &str, detect: &str) -> SettingAdapter {
    SettingAdapter {
        name: name.into(),
        detect: detect.into(),
        os: None,
        read: vec![
            detect.into(),
            "read".into(),
            "{schema}".into(),
            "{key}".into(),
        ],
        write: vec![
            detect.into(),
            "write".into(),
            "{schema}".into(),
            "{key}".into(),
            "{value}".into(),
        ],
        reset: vec![
            detect.into(),
            "reset".into(),
            "{schema}".into(),
            "{key}".into(),
        ],
        system_read: vec![],
        system_write: vec![],
        system_reset: vec![],
    }
}
fn registry_like() -> SettingAdapter {
    let mut r = row("winreg", "reg");
    r.system_read = vec!["reg".into(), "query".into(), "HKLM/{schema}".into()];
    r.system_write = vec!["reg".into(), "add".into(), "HKLM/{schema}".into()];
    r.system_reset = vec!["reg".into(), "delete".into(), "HKLM/{schema}".into()];
    r
}
fn store_reading(reading: &str) -> SettingBackendCore {
    #[cfg(windows)]
    let read = vec!["cmd".into(), "/C".into(), format!("echo {}", reading)];
    #[cfg(not(windows))]
    let read = vec![
        "sh".into(),
        "-c".into(),
        format!("printf '%s' '{}'", reading),
    ];
    let mut a = row("probe", if cfg!(windows) { "cmd" } else { "sh" });
    a.read = read;
    SettingBackendCore::new(CommandExecutor::new(false, false), vec![a])
}
fn registry() -> SettingAdapter {
    adapters(vec![])
        .into_iter()
        .find(|a| a.name == "windows-registry")
        .expect("the registry row must ship")
}
mod a_command_driven_store_is_addressed_by_where_it_runs_and_a_registry_one_by_its_argv;
mod a_different_value_is_not_already_set;
mod a_key_the_store_already_holds_reads_back_as_in_effect;
mod a_machine_wide_scope_named_at_one_account_is_refused;
mod a_machine_with_no_store_and_a_name_with_no_key_are_both_unanswerable;
mod a_quoted_reading_equals_the_bare_value;
mod a_read_is_asked_in_the_account_the_line_named;
mod a_read_the_store_refuses_is_unanswerable_not_absent;
mod a_reg_query_block_is_read_as_its_value;
mod a_row_for_another_os_is_not_this_machines_store;
mod a_row_missing_a_command_is_refused_rather_than_half_used;
mod a_store_that_cannot_address_a_named_account_refuses_before_writing;
mod a_store_with_both_scopes_runs_different_commands;
mod a_store_with_no_compiled_in_support_is_driven_from_a_row;
mod a_store_without_system_scope_reports_it;
mod a_teardown_of_a_contradictory_row_is_refused_before_it_resets_anything;
mod a_user_row_cannot_take_a_shipped_name;
mod gsettings_deliberately_gets_no_option_terminator;
mod gsettings_reads_writes_and_resets_a_key;
mod system_scope_on_a_user_only_store_is_unanswerable;
mod the_refusal_names_the_stores_it_looked_for;
mod the_registry_row_addresses_a_hive_rather_than_a_literal_hkcu;
mod the_shipped_table_parses_and_carries_gsettings;
mod writing_the_default_scope_is_accepted;
