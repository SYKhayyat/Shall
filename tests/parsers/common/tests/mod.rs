pub use shall::core::Package;
pub use shall::parsers::common::*;
pub use shall::parsers::{or_unrecognised, ParseResult};
pub use shall::utils::text::sanitize;
mod a_dash_in_the_name_is_not_mistaken_for_a_version;
mod a_name_with_no_version_survives_whole;
mod apk_search_output_yields_the_bare_name;
