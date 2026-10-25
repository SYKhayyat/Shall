pub use shall::core::Package;
pub use shall::parsers::bsd::*;
pub use shall::parsers::{or_unrecognised, ParseResult};
pub use shall::utils::text::sanitize;
mod a_line_that_parses_as_nothing_is_reported_rather_than_read_as_an_empty_machine;
mod freebsd_pkg_info_names_and_versions;
mod genuinely_empty_output_is_an_empty_machine;
mod openbsd_pkg_info_names_and_versions_carry_the_pkg_add_label;
