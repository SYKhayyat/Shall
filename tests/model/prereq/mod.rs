use serde::Deserialize;
pub use shall::core::adapter::{self, AdapterRow};
pub use shall::model::prereq::*;

fn row(manager: &str, run: &[&str]) -> PrereqDef {
    PrereqDef {
        manager: manager.into(),
        os: None,
        missing: "something".into(),
        probe: vec!["probe".into()],
        probe_output: None,
        run: run.iter().map(|s| (*s).to_string()).collect(),
    }
}
mod a_row_is_per_package_when_its_command_names_the_package;
mod a_row_that_cannot_answer_is_refused;
mod an_output_probe_is_line_exact;
mod rows_are_selected_by_manager_and_os;
mod the_builtin_rows_are_usable;
