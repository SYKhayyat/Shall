pub use serde::{Deserialize, Serialize};
pub use shall::config::settings::*;
pub use shall::core::{Error, Result};
pub use std::path::{Path, PathBuf};

fn at(name: &str) -> PathBuf {
    PathBuf::from(name)
}
fn absolute(tail: &str) -> String {
    if cfg!(windows) {
        format!("C:/{}", tail)
    } else {
        format!("/{}", tail)
    }
}
mod a_relative_root_is_refused;
mod a_second_key_is_refused_and_names_where_it_belongs;
mod an_absent_settings_file_is_not_an_error;
mod an_empty_settings_file_is_valid_and_means_the_default;
mod every_source_can_say_what_it_was;
mod the_flag_outranks_everything_stored;
mod the_one_key_is_read;
mod the_settings_file_is_never_inside_the_repo_it_locates;
mod the_settings_file_outranks_the_default;
mod with_nothing_set_the_source_is_the_default;
