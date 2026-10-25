use once_cell::sync::Lazy;
use regex::Regex;
pub use shall::parsers::utils::*;
mod a_line_with_no_brackets_has_no_version_rather_than_all_of_it;
mod a_version_is_taken_from_either_bracket;
mod an_unquoted_line_splits_the_ordinary_way;
mod test_split_columns_quoted;
