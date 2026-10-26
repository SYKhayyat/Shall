pub use dialoguer::{theme::ColorfulTheme, Select};
pub use shall::app::snapshot_restore::*;
pub use shall::core::{Error, ManagedPackage, Result, Snapshot, SnapshotManager, StateRegistry};
use std::collections::HashMap;
pub use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::Mutex;
pub use tracing::{debug, info, warn};

fn reg(pkgs: &[(&str, &str, &str)]) -> StateRegistry {
    let mut r = StateRegistry::default();
    for (backend, name, version) in pkgs {
        r.add(
            backend,
            name,
            Some(version.to_string()),
            Default::default(),
            "imperative",
            false,
        );
    }
    r
}
mod diff_reports_added_removed_and_version_changed;
mod identical_states_produce_an_empty_diff;

mod snapshot_restore_reaches_every_provider_tests;
