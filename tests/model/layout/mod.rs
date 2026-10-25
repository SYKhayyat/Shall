pub use shall::model::layout::*;
pub use std::path::{Path, PathBuf};

fn layout() -> Layout {
    Layout::new("/cfg", "/data")
}
mod a_module_file_is_its_name_lowercased;
mod a_module_name_cannot_climb_out_of_the_modules_folder;
mod a_module_name_is_lowercased_once_at_construction;
mod shall_data_is_never_inside_your_repo;
mod your_repo_holds_what_you_wrote;
