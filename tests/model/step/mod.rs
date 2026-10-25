use serde::Deserialize;
pub use shall::core::adapter::{AdapterRow, Detected};
pub use shall::model::step::Catalogue;
pub use shall::model::step::*;
mod a_step_belongs_to_the_platforms_its_row_names;
mod a_step_runs_as_argv_and_not_through_a_shell;
mod every_shipped_step_has_its_own_fingerprint;
mod only_the_reserved_prefix_names_a_step;
mod the_shipped_catalogue_parses_and_every_row_is_usable;
