pub use shall::core::{Error, Result};
pub use shall::utils::file::*;
use std::fs;
use std::io::Write;
pub use std::path::{Component, Path, PathBuf};
use tempfile::NamedTempFile;

fn dir() -> PathBuf {
    PathBuf::from("/home/u/.local/bin")
}
mod a_drive_relative_bin_name_is_refused;
mod a_plain_name_lands_in_the_bin_directory;
mod a_reserved_device_name_is_refused;
mod a_traversing_bin_name_is_refused;
mod a_url_names_its_last_path_segment_and_nothing_else;
mod a_url_that_names_no_file_is_refused;
mod the_guard_key_is_what_opens_it;
