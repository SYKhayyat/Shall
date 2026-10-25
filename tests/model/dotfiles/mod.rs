pub use shall::model::dotfiles::*;
pub use std::collections::{BTreeMap, BTreeSet};
pub use std::path::{Path, PathBuf};

fn tree_with(files: &[&str]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for f in files {
        let path = dir.path().join(f);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "x").unwrap();
    }
    dir
}
fn nothing_exists(_: &Path) -> bool {
    false
}
fn nothing_owned(_: &Path) -> bool {
    false
}
mod a_destination_the_user_owns_is_a_collision_and_ours_is_not;
mod a_fresh_machine_has_no_collisions;
mod an_encrypted_file_is_placed_as_ciphertext_like_any_other;
mod destinations_mirror_the_tree_under_the_target_root;
mod the_plan_is_in_a_stable_order;
mod the_tree_places_files_not_directories;
mod the_trees_own_git_metadata_is_never_placed;
mod trees_that_do_not_overlap_are_not_a_conflict;
mod two_trees_claiming_one_destination_are_named_together;
