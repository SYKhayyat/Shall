use async_trait::async_trait;
pub use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
pub use serde::{Deserialize, Serialize};
pub use shall::config::Config;
pub use shall::core::adapter::{self, AdapterRow, Detected};
pub use shall::core::snapshot::*;
pub use shall::core::{CommandExecutor, Error, Result};
use std::path::Path;
use std::process::Command as StdCommand;
pub use tracing::{info, warn};

fn btrfs_id(local: DateTime<Local>) -> String {
    format!("shall_pre_pre_sync_{}", local.format("%Y%m%d%H%M%S"))
}
fn zfs_id(local: DateTime<Local>) -> String {
    format!("tank/root@shall_{}", local.format("%Y%m%d_%H%M%S"))
}
fn snap(id: &str, description: &str, backend: &str) -> Snapshot {
    Snapshot {
        id: id.into(),
        timestamp: Utc::now().to_rfc3339(),
        description: description.into(),
        backend: backend.into(),
    }
}
fn def(name: &str) -> SnapshotProviderDef {
    SnapshotProviderDef {
        name: name.into(),
        os: None,
        detect: "true".into(),
        source: "tank/root".into(),
        create: vec!["mk".into(), "{id}".into(), "{source}".into()],
        list: vec!["ls".into()],
        delete: vec!["rm".into(), "{id}".into()],
        restore: vec![],
        restores_running_system: false,
        list_pattern: r"(\S+)".into(),
        restore_how: None,
        detect_path: None,
        id_template: None,
        create_id_pattern: None,
        list_needs_root: false,
        powershell: false,
    }
}
struct Fake {
    name: String,
    here: bool,
}
#[async_trait]
impl SnapshotProvider for Fake {
    fn name(&self) -> &str {
        &self.name
    }
    async fn is_available(&self) -> bool {
        self.here
    }
    async fn create(&self, _l: SnapshotLabel) -> Result<Snapshot> {
        unreachable!()
    }
    async fn list(&self) -> Result<Vec<Snapshot>> {
        Ok(vec![])
    }
    async fn delete(&self, _id: &str) -> Result<()> {
        Ok(())
    }
    async fn restore(&self, _id: &str) -> Result<()> {
        Ok(())
    }
    fn restore_capability(&self) -> RestoreCapability {
        RestoreCapability::Live
    }
}
fn fake(name: &str, here: bool) -> Box<dyn SnapshotProvider> {
    Box::new(Fake {
        name: name.into(),
        here,
    })
}
mod a_config_provider_is_create_only_unless_it_declares_both;
mod a_config_provider_missing_a_required_command_is_refused;
mod a_create_only_config_provider_refuses_restore_and_names_the_snapshot;
mod a_create_only_config_provider_reports_not_from_running_system;
mod a_parsed_snapshot_reads_its_real_age_not_zero;
mod a_priority_that_names_nothing_present_still_falls_back;
mod a_restore_point_id_that_is_not_a_number_never_reaches_powershell;
mod a_user_made_snapshot_is_not_owned_and_is_left_alone;
mod an_id_with_no_embedded_time_returns_none;
mod an_older_id_parses_to_an_earlier_time_than_a_newer_one;
mod btrfs_id_round_trips_to_its_creation_time;
mod every_snapshot_label_is_a_fixed_string;
mod no_priority_keeps_registration_order;
mod ownership_is_recognized_across_every_provider;
mod priority_picks_the_first_available_in_the_declared_order;
mod priority_skips_a_named_provider_that_is_absent;
mod the_builtin_snapshot_defs_hold_their_invariants;
mod the_snapshot_provider_schema_parses;
mod the_windows_row_substitutes_the_id_only_as_a_number;
mod zfs_id_round_trips_to_its_creation_time;
