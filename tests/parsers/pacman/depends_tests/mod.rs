pub use shall::core::Package;
pub use shall::parsers::pacman::parse_depends_on;
pub use shall::parsers::pacman::*;
pub use shall::parsers::{or_unrecognised, ParseResult};
pub use shall::utils::text::sanitize;

const SI_JQ: &str = "\
Repository      : extra
Name            : jq
Version         : 1.8.1-1
Depends On      : glibc  oniguruma
Optional Deps   : None
Conflicts With  : None
Description     : Command-line JSON processor
";
mod a_depends_row_is_matched_by_its_key_and_not_by_its_column;
mod a_package_with_no_dependencies_reports_none_of_them;
mod a_row_of_the_same_shape_that_is_not_a_dependency_is_left_alone;
mod a_version_constraint_is_not_part_of_the_name;
