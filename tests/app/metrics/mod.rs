pub use chrono::{DateTime, Utc};
pub use serde::{Deserialize, Serialize};
pub use shall::app::metrics::*;
pub use std::sync::{Arc, Mutex};
use std::time::Instant;

fn op(backend: &str, ms: u64) -> OperationMetrics {
    OperationMetrics {
        name: "x".into(),
        backend: backend.into(),
        started_at_unix: 0,
        duration_ms: ms,
        success: true,
        error: None,
        retry_count: 0,
        bytes_downloaded: 0,
        batch_size: 1,
    }
}
mod a_rebuild_never_calls_its_removals_removals;
mod a_recorded_failure_is_a_failed_run_and_nothing_has_to_be_told_twice;
mod rollup_of_nothing_is_empty;
mod rollup_sums_and_tracks_slowest_per_backend;
mod the_counters_hold_only_what_they_were_given;
