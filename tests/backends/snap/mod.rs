use async_trait::async_trait;
pub use shall::backends::snap::*;
pub use shall::core::{
    BackendCore, CommandExecutor, Error, Installable, MetadataProvider, Package, PackageSpec,
    Queryable, Result, Searchable, Upgradable,
};
pub use shall::utils::text::sanitize;
use std::sync::Arc;
pub use tracing::{debug, info, warn};

fn snap_with(
    pattern: &str,
    output: &str,
) -> (
    Arc<SnapBackendCore>,
    Arc<shall::core::executor::MockExecutor>,
) {
    let vfs = Arc::new(dashmap::DashMap::new());
    let mock = Arc::new(shall::core::executor::MockExecutor::new(vfs.clone()));
    mock.set_response(
        pattern,
        Ok(shall::core::executor::DryRunOutput {
            stdout: output.as_bytes().to_vec(),
            stderr: vec![],
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
    (Arc::new(SnapBackendCore::new(exec)), mock)
}
fn spec_with(name: &str, options: &[(&str, &str)]) -> PackageSpec {
    PackageSpec {
        name: name.to_string(),
        backend: "snap".to_string(),
        options: options
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
        ..Default::default()
    }
}
mod a_batch_of_snaps_is_one_remove_call;
mod a_first_install_still_asks_for_classic;
mod a_refresh_carries_every_switch_the_declaration_asks_for;
mod a_snap_named_like_a_flag_stays_a_name;
mod a_snaps_state_is_read_from_the_one_report;
mod info_answers_installed_here_not_present_in_the_store;
mod moving_to_a_more_stable_channel_is_a_downgrade;
mod snap_channel_reaches_the_command;
mod snap_find_parses_rows;
mod snap_without_a_channel_passes_no_channel_flag;
mod snaps_other_name_carrying_commands_terminate_too;
