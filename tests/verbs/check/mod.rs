pub use shall::core::{HealthReport, HealthStatus};
pub use shall::verbs::check::*;
pub use shall::verbs::inventory::handle_audit;
pub use shall::verbs::plan::handle_status;
pub use shall::verbs::prelude::*;

fn rep(status: HealthStatus) -> HealthReport {
    HealthReport {
        status,
        message: None,
    }
}
mod a_machine_with_nothing_wrong_has_no_criticals;
mod a_missing_program_is_named_by_the_program_that_was_probed;
mod status_labels_are_stable;
mod tally_counts_each_status;
