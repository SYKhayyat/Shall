pub use shall::config::grammar::error::{GrammarError, Origin, Result};
pub use shall::config::grammar::options::*;
use std::collections::BTreeMap;

fn o() -> Origin {
    Origin::new("modules/dev.txt", 3)
}
mod a_bare_key_is_true;
mod a_bare_version_says_what_was_meant;
mod a_block_line_without_an_equals_is_an_error;
mod a_block_value_keeps_a_hash_that_is_part_of_the_value;
mod a_block_value_runs_verbatim_to_end_of_line;
mod a_block_value_that_looks_like_it_has_a_comment_says_so;
mod a_comma_in_a_value_is_an_error_not_a_guess;
mod a_doubled_comma_is_an_error;
mod a_repeated_key_makes_a_list;
mod a_second_option_written_with_a_space_is_refused_rather_than_swallowed;
mod an_at_sign_inside_a_value_is_not_two_options;
mod an_empty_option_list_is_an_error;
mod an_option_with_nothing_after_the_equals_is_refused;
mod every_error_names_the_file_and_line;
mod short_form_reads_key_value_pairs;
mod short_form_reads_several_pairs;
