pub use shall::core::Package;
pub use shall::parsers::macos::*;
pub use shall::parsers::{or_unrecognised, ParseResult};
pub use shall::utils::text::sanitize;
mod test_macports_installed_parsing;
mod test_macports_search_parsing;
mod test_mas_list_parsing;
mod test_mas_search_parsing;
