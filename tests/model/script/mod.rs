pub use shall::core::{Error, Result};
pub use shall::model::script::*;
use std::path::Path;

fn shebang(contents: &str) -> Option<Shebang> {
    shebang_of(contents).expect("parses")
}
fn an_interpreter_that_exists() -> String {
    std::env::current_exe()
        .expect("the test binary has a path")
        .to_string_lossy()
        .into_owned()
}
mod a_carriage_return_is_not_part_of_the_interpreter;
mod a_missing_interpreter_names_every_spelling_that_was_tried;
mod a_path_holding_an_equals_sign_is_not_an_assignment;
mod a_path_with_a_space_stays_one_argument;
mod a_plain_shebang_names_its_interpreter;
mod a_script_with_no_shebang_gets_the_platform_default;
mod a_shebang_indented_in_a_toml_block_is_still_a_shebang;
mod an_absolute_interpreter_that_exists_is_taken_as_written;
mod an_environment_assignment_is_refused_rather_than_half_honoured;
mod env_is_stripped_because_the_lookup_happens_here;
mod python3_falls_back_to_the_windows_spellings;
mod the_interpreters_own_arguments_survive;
mod the_name_is_the_last_segment_whichever_slash_was_used;
mod the_script_is_the_last_argument;
mod the_suffix_is_what_this_platforms_interpreter_accepts;
mod what_is_not_a_shebang;
