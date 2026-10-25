pub use shall::backends::artifact::discover::*;
use std::fmt;
pub use std::path::{Path, PathBuf};

fn entries(paths: &[(&str, bool)]) -> Vec<Entry> {
    paths.iter().map(|(p, x)| Entry::new(*p, *x)).collect()
}
mod a_lone_executable_under_a_different_name_is_accepted;
mod a_man_page_is_not_the_program;
mod a_windows_exe_is_the_same_program;
mod an_empty_archive_says_so;
mod an_exact_name_wins_over_a_sibling_that_merely_starts_with_it;
mod bin_also_accepts_a_bare_filename;
mod bin_names_the_executable_by_path;
mod bin_turns_the_guess_off_rather_than_falling_back_to_it;
mod the_common_case_needs_no_option;
mod the_name_match_outranks_the_lone_executable_rule;
mod the_same_name_twice_is_an_error_rather_than_a_guess;
mod two_lone_executables_with_unrelated_names_are_ambiguous;
