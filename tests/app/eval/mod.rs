use serde::Serialize;
pub use shall::app::eval::*;
pub use shall::config::grammar::Origin;
use std::path::Path;

fn spec(backend: &str, name: &str, present: bool) -> shall::core::PackageSpec {
    let mut options = shall::config::grammar::Options::default();
    options.set("__source", format!("/repo/modules/x.txt:{}", name.len()));
    shall::core::PackageSpec {
        name: name.into(),
        backend: backend.into(),
        options,
        present,
        ..Default::default()
    }
}
mod a_drive_letter_is_not_mistaken_for_a_line_number;
mod a_source_outside_the_repo_stays_absolute;
mod an_empty_configuration_evaluates_to_an_empty_document;
mod every_package_names_the_line_that_declared_it;
mod extras_split_into_kind_and_subject;
mod lapsed_lines_are_reported;
mod present_and_absent_are_separate_and_sorted;
mod sources_are_repo_relative;
mod the_document_is_versioned;
mod the_rendered_document_ends_in_a_newline;
mod the_same_state_serialises_identically_twice;
mod variables_keep_their_types;
mod windows_sources_come_back_with_forward_slashes;
