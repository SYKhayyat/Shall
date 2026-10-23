pub use shall::core::Package;
pub use shall::parsers::pacman::*;
pub use shall::parsers::{or_unrecognised, ParseResult};
pub use shall::utils::text::sanitize;
mod test_pacman_list_parsing;
mod test_pacman_search_parsing;
