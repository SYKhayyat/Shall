pub use shall::core::Package;
pub use shall::parsers::dnf::parse_bare_dependency_names;
pub use shall::parsers::dnf::*;
pub use shall::parsers::{or_unrecognised, ParseResult};
pub use shall::utils::text::sanitize;
mod a_package_with_nothing_to_report_reports_nothing;
mod one_name_per_line_and_blank_lines_are_not_packages;
