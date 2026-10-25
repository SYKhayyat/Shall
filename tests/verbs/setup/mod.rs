pub use shall::app::Backends;
pub use shall::core::CommandExecutor;
pub use shall::verbs::perform_maintenance;
pub use shall::verbs::plan::{
    approve_adapters, approve_exec_scripts, approve_generate_commands, build_and_write_locks,
    resolve_for_approval,
};
pub use shall::verbs::prelude::*;
pub use shall::verbs::setup::*;
pub use shall::verbs::sync::{enforce_policy, print_flight_plan};
mod answers_layer_onto_config;
mod config_from_answers_round_trips_through_toml;
mod config_template_actually_parses_and_matches_the_defaults;
mod omitted_snapshot_count_keeps_base_default;
mod the_shipped_example_parses_too;
mod the_template_documents_no_setting_that_would_disarm_the_guard;
