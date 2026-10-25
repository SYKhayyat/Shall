pub use serde::{Deserialize, Serialize};
pub use shall::core::state::*;
pub use shall::core::{Error, Result};
pub use shall::utils::file::persist;
pub use shall::utils::safe_data_dir;
use std::collections::BTreeMap;
pub use std::path::{Path, PathBuf};
pub use std::time::{SystemTime, UNIX_EPOCH};
pub use tracing::{debug, info, trace};

fn reg() -> StateRegistry {
    StateRegistry::new(PathBuf::from("/tmp/shall-test-registry.json"))
}
mod a_malformed_duration_is_refused_rather_than_fatal;
mod a_registry_from_before_a_field_existed_loads_with_the_field_empty;
mod a_row_with_no_origin_is_refused_rather_than_read;
mod a_well_formed_duration_is_that_many_seconds_from_now;
mod clear_suspension_removes_only_the_target;
mod due_suspensions_are_only_past_deadline;
mod every_managed_row_carries_the_origin_it_was_given;
mod hold_matches_qualified_and_bare_names;
mod invalid_duration_is_rejected;
mod resuspending_replaces_prior_record;
mod session_scoped_never_appears_in_timed_sweep;
mod session_scoped_suspension_binds_active_session;
mod timed_suspension_sets_future_restore_and_no_session;
