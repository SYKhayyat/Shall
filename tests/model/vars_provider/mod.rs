pub use shall::config::grammar::{GrammarError, Origin, Result};
pub use shall::config::parser::HostFacts;
pub use shall::model::vars::{Value, VarOrigins, Vars};
pub use shall::model::vars_provider::*;
pub use std::path::{Path, PathBuf};

fn tmp() -> tempfile::TempDir {
    tempfile::tempdir().expect("a temp dir")
}
fn facts() -> HostFacts {
    HostFacts {
        os: "linux".into(),
        arch: "x86_64".into(),
        host: "laptop".into(),
        family: "debian".into(),
        home: None,
        user: None,
        vars: Default::default(),
    }
}
mod a_bad_variable_name_is_refused;
mod a_banner_before_the_document_does_not_either;
mod a_byte_order_mark_does_not_turn_json_into_pairs;
mod a_json_object_value_is_refused;
mod a_named_source_picks_that_one_and_names_its_kind;
mod a_named_source_that_is_absent_is_an_error;
mod a_named_source_with_a_nonsense_name_is_an_error;
mod a_pairs_line_whose_value_is_json_is_still_a_pairs_line;
mod a_provider_that_exits_nonzero_is_an_error_with_its_stderr;
mod a_py_provider_finds_this_machines_python_whatever_it_is_called;
mod a_sole_provider_is_selected_without_a_source_key;
mod an_external_provider_runs_and_its_output_is_parsed;
mod an_unknown_extension_names_no_interpreter;
mod json_object_output_carries_its_types;
mod name_value_lines_infer_types_like_a_vars_line;
mod no_provider_file_is_no_variables;
mod pairs_and_empty_output_are_unchanged;
mod resolving_the_interpreter_keeps_the_flags_that_follow_it;
mod two_providers_and_no_choice_is_a_loud_error;
mod vars_shall_is_the_embedded_kind;
