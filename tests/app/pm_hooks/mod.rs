pub use shall::app::pm_hooks::*;
use std::path::PathBuf;

fn s(v: &[&str]) -> Vec<String> {
    v.iter().map(|x| x.to_string()).collect()
}
mod detect_operation_ignores_read_only;
mod detect_operation_long_form;
mod detect_operation_pacman_short_flags;
mod diff_installed_finds_added_and_removed;
mod diff_installed_no_change;
mod extract_targets_does_not_adopt_a_flag_s_value;
mod extract_targets_drops_binary_flags_and_keyword;
mod hook_op_round_trips;
mod hook_specs_reference_the_binary_and_cover_many_managers;
mod local_file_detection_by_path_and_extension;
mod local_file_stem_strips_dir_and_extension;
mod pacman_hook_passes_targets_and_sets_operation;
mod shell_wrappers_wrap_known_managers_and_add_learn_helper;
