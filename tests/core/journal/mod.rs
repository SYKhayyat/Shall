pub use chrono::{Duration as ChronoDuration, Utc};
pub use serde::{Deserialize, Serialize};
pub use shall::core::journal::*;
pub use shall::core::{Error, PackageSpec, Result};
pub use shall::utils::file::persist;
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::tempdir;
pub use tracing::{debug, info, trace, warn};
use uuid::Uuid;

fn a_spec(name: &str) -> shall::core::PackageSpec {
    shall::core::PackageSpec {
        name: name.to_string(),
        backend: "apt".to_string(),
        options: Default::default(),
        requires: Vec::new(),
        present: true,
    }
}
mod a_completed_install_is_recorded_even_though_it_is_not_interrupted;
mod a_completed_mutation_leaves_nothing_to_heal;
mod a_corrupt_wal_does_not_brick_every_command;
mod a_failed_entry_ages_out_and_a_half_done_one_never_does;
mod a_failed_mutation_is_recorded_failed_and_not_healable;
mod a_log_that_cannot_be_written_refuses_the_mutation;
mod a_missing_wal_starts_fresh_without_error;
mod an_abandoned_mutation_stays_interrupted;
mod an_aged_out_crash_is_still_healable;
mod recovery_finishes_interrupted_work_and_does_not_retry_failures;
mod the_record_reaches_disk_before_the_mutation_runs;
