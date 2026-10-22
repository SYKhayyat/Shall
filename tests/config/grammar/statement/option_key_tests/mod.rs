use super::*;

pub use shall::config::grammar::statement::*;
fn known(name: &str) -> bool {
    matches!(name, "apt" | "cargo" | "winget" | "npm")
}
fn parse_line(line: &str) -> Result<Statement> {
    parse(&Origin::new("modules/dev.txt", 1), line, &known)
}

mod a_link_with_a_windows_path_is_a_link_not_an_expression;
mod a_name_may_open_with_an_at_sign_and_still_take_options;
mod a_package_name_carrying_a_backslash_is_a_package_not_a_difference;
mod an_ordinary_name_still_takes_its_options_at_the_first_at_sign;
mod an_unknown_key_lists_the_real_ones;
mod every_key_in_the_table_is_accepted;
mod lease_is_refused_and_points_at_the_dated_line;
mod no_punctuation_turns_a_qualified_line_into_set_math;
mod set_math_between_qualified_packages_is_still_set_math;
mod until_on_a_present_line_is_refused;
