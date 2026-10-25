pub use shall::model::cache::*;
use std::collections::HashSet;
pub use std::path::{Path, PathBuf};
use tempfile::tempdir;
mod a_directory_named_like_the_artifact_is_never_deleted;
mod an_empty_basename_matches_nothing;
mod an_exact_filename_at_depth_is_found_and_a_near_miss_is_not;
