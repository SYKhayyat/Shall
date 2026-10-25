pub use shall::core::Package;
pub use shall::parsers::bsd::*;
pub use shall::parsers::{or_unrecognised, ParseResult};
pub use shall::utils::text::sanitize;
mod a_bare_pkgver_listing_reads_the_same_way;
mod a_dependency_constraint_is_not_part_of_the_name;
mod a_search_carries_its_install_marker_and_its_description;
mod a_state_flag_is_not_part_of_the_package;
