pub use shall::core::Package;
pub use shall::parsers::dnf::*;
pub use shall::parsers::{or_unrecognised, ParseResult};
pub use shall::utils::text::sanitize;
mod dnf5_tab_separated_output_is_read;
mod only_a_real_architecture_is_stripped_from_a_name;
mod test_dnf_search_parsing;
mod test_rpm_qa_parsing;
mod zypper_rows_are_read_with_or_without_the_header_rule;
