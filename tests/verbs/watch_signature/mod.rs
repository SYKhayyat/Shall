pub use sha2::{Digest, Sha256};
pub use shall::verbs::watch_signature::*;
use std::collections::BTreeMap;
use std::path::Path;
use tempfile::tempdir;

async fn repo() -> tempfile::TempDir {
    let dir = tempdir().expect("tempdir");
    for sub in ["modules", "profiles", "vars", "adapters", "locks"] {
        std::fs::create_dir_all(dir.path().join(sub)).expect("the tree");
    }
    std::fs::write(dir.path().join("modules/dev.txt"), "apt:htop\n").expect("a manifest");
    std::fs::write(dir.path().join("active"), "Work\n").expect("active");
    std::fs::write(dir.path().join("preferences.toml"), "").expect("preferences");
    std::fs::write(dir.path().join("locks/regex.toml"), "").expect("a lock");
    dir
}
mod a_line_that_is_not_a_pattern_says_why;
mod a_pattern_is_anchored_only_when_it_says_so;
mod a_removed_directory_names_what_it_held;
mod a_removed_file_is_a_change;
mod a_rewrite_of_the_same_bytes_at_the_same_length_is_a_change;
mod a_symlink_is_hashed_as_the_pointer;
mod an_edit_anywhere_in_the_repo_is_a_change;
mod an_unreadable_file_is_skipped_rather_than_fatal;
mod the_excluded_directory_is_the_one_the_layout_creates;
mod the_ignore_file_is_the_users_own_list;
mod the_line_names_what_it_reacted_to_without_becoming_a_list;
mod the_same_bytes_rewritten_is_not_a_change;
mod two_passes_over_an_unchanged_repo_are_equal;
mod what_shall_writes_and_git_does_is_not_a_change;
